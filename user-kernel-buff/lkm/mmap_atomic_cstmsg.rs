// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

mod vallocator;
mod timestamp;
use alloc::vec::Vec;
use core::{
    slice,
    sync::atomic::{fence, AtomicBool, AtomicUsize, Ordering},
    // time::Duration,
    u8,
};
use kernel::bindings;
use kernel::prelude::*;
use kernel::{
    user_ptr::{UserSlicePtrWriter, UserSlicePtr},
    // delay,
    io_buffer::IoBufferWriter,
    file::{File, Operations, IoctlHandler, IoctlCommand},
    miscdev,
    mm::virt::Area,
    str::CString,
    sync::{Arc, ArcBorrow, UniqueArc},
    task::Task,
};
use vallocator::VAllocator;
use timestamp::*;
module! {
    type: RustMiscdev,
    name: "rustdev",
    author: "Rust for Linux Contributors",
    description: "Rust miscellaneous device sample",
    license: "GPL",
    params: {
        msg_size: usize {
            default: 4096,
            permissions: 0,
            description: "",
        },
        msg_slots: usize {
            default: 8,
            permissions: 0,
            description: "",
        },
        buf_offset: usize {
            default: 4096,
            permissions: 0,
            description: "",
        },
    },
}

struct Producer {
    stop: AtomicBool,
    headp: *const AtomicUsize,
    tailp: *const AtomicUsize,
    mmaped: Vec<u8, VAllocator>,
    recorder: Vec<Timestamp, VAllocator>,
    ts_trans_cnt: AtomicUsize,
}

unsafe impl Sync for Producer {}
unsafe impl Send for Producer {}

impl IoctlHandler for Producer {
    type Target<'a> = &'a Self;

    // transfer the number of timestamps
    fn read(
        this: Self::Target<'_>,
        _file: &File,
        cmd: u32,
        writer: &mut UserSlicePtrWriter,
    ) -> Result<i32> {
        match cmd & bindings::_IOC_NRMASK {
            0 => {
                let len = this.recorder.len() as u32;
                this.ts_trans_cnt.store(len as _, Ordering::SeqCst);
                writer.write_slice(len.to_ne_bytes().as_slice()).unwrap();
                Ok(0)
            }
            _ => panic!("unknown command"),
        }
    }

    // transfer raw timstamps
    fn pure(this: Self::Target<'_>, _file: &File, cmd: u32, arg: usize) -> Result<i32> {
        match cmd & bindings::_IOC_NRMASK {
            1 => {
                let mut raw_vec = Vec::<u8, VAllocator>::new_in(VAllocator);
                this.recorder.iter().for_each(|x| {
                    raw_vec.try_extend_from_slice(x.into_raw().as_slice()).unwrap()
                });
                
                let len = this.ts_trans_cnt.load(Ordering::SeqCst) * 18;
                let user_ptr = unsafe { UserSlicePtr::new(arg as _, len) };
                let mut writer = user_ptr.writer();
                
                writer.write_slice(&raw_vec[0..len]).unwrap();
                
                Ok(0)
            }
            _ => panic!("unknown command"),
        }
    }
}

impl Producer {
    fn try_new() -> Result<Arc<Producer>> {
        let len = *msg_size.read() * *msg_slots.read() + *buf_offset.read();
        let mut mmaped = Vec::try_with_capacity_in(len, VAllocator).unwrap();
        unsafe { mmaped.set_len(len) };
        let headp = mmaped.as_ptr() as *const AtomicUsize;
        let tailp = unsafe { headp.add(1) };

        let pinned = Pin::from(UniqueArc::try_new(Self {
            stop: AtomicBool::new(false),
            headp,
            tailp,
            mmaped,
            recorder: Vec::new_in(VAllocator),
            ts_trans_cnt: AtomicUsize::new(0),
        })?);
        Ok(pinned.into())
    }

    fn reset(&self) {
        self.stop.store(false, Ordering::SeqCst);
        unsafe {
            (*self.headp).store(0, Ordering::SeqCst);
            (*self.tailp).store(0, Ordering::SeqCst);
            let ptr = &self.recorder as *const Vec<Timestamp, VAllocator> as *mut Vec<Timestamp, VAllocator>;
            (*ptr).clear();
        }
    }

    fn head_ref(&self) -> &AtomicUsize {
        unsafe { &*self.headp }
    }

    fn tail_ref(&self) -> &AtomicUsize {
        unsafe { &*self.tailp }
    }

    #[allow(dead_code)]
    fn rcder_mut_ref(&self) -> &mut Vec<Timestamp, VAllocator> {
        unsafe {
            &mut *(&self.recorder as *const Vec<Timestamp, VAllocator>).cast_mut()
        }
    }
}

#[vtable]
impl Operations for Producer {
    type Data = Arc<Producer>;
    type OpenData = Arc<Producer>;

    fn open(this: &Self::OpenData, _file: &File) -> Result<Self::Data> {
        let mut ticker = 0;
        let shared = this.clone();
        shared.reset();
        let offset = *buf_offset.read();
        let slots = *msg_slots.read();
        let msgsiz = *msg_size.read();
        let msg = CString::try_from_fmt(fmt!("{0:>1$}", -1, msgsiz-1)).unwrap();
        let _ = Task::spawn(fmt!("producer"), move || loop {
            {
                let len = msg.len_with_nul();
                let rcder = shared.rcder_mut_ref();

                let head;
                let next_head;
                let slot;
                let mut tail;

                // -------- timestamp: start sync for write --------
                rcder.try_push(Timestamp::new(TimestampTag::WriteSyncStart, ticker)).unwrap();
                {
                    // get head & tail
                    head = shared.head_ref().load(Ordering::SeqCst);
                    tail = shared.tail_ref().load(Ordering::SeqCst);
                    next_head = (head + 1) % slots;

                    // it is the slot which we will write into
                    let start = offset + msgsiz * head;
                    let end = start + len;
                    slot = unsafe { 
                        let slot = &shared.mmaped[start..end];
                        slice::from_raw_parts_mut(slot.as_ptr().cast_mut(), slot.len())
                    };

                    // spin until user increase tail
                    while next_head == tail {
                        if shared.stop.load(Ordering::Relaxed) {
                            pr_info!("exit normally\n");
                            return;
                        }

                        tail = shared.tail_ref().load(Ordering::SeqCst);
                    }
                }
                // -------- timestamp: end sync for write --------
                rcder.try_push(Timestamp::new(TimestampTag::WriteSyncEnd, ticker)).unwrap();

                // -------- timestamp: start write --------
                rcder.try_push(Timestamp::new(TimestampTag::WriteStart, ticker)).unwrap();
                {
                    // write messssage
                    slot.copy_from_slice(msg.as_bytes_with_nul());
                }
                // -------- timestamp: end write --------
                rcder.try_push(Timestamp::new(TimestampTag::WriteEnd, ticker)).unwrap();

                ticker += 1;
                // make sure `write` will be finished before increase head
                fence(Ordering::SeqCst);

                // increase head
                shared.head_ref().store(next_head, Ordering::SeqCst);
            }
        });
        Ok(this.clone())
    }

    fn ioctl(data: ArcBorrow<'_, Producer>, file: &File, cmd: &mut IoctlCommand) -> Result<i32> {
        let _ = cmd.dispatch::<Self>(&data, file);
        Ok(1)
    }

    fn mmap(data: ArcBorrow<'_, Producer>, _file: &File, vma: &mut Area) -> Result {
        pr_info!("mmaping...\n");
        let vma_ptr_ptr = vma as *const Area as *const *mut bindings::vm_area_struct;
        unsafe { bindings::remap_vmalloc_range(*vma_ptr_ptr, data.mmaped.as_ptr() as _, 0) };

        Ok(())
    }

    fn release(data: Self::Data, _file: &File) {
        data.stop.store(true, Ordering::Relaxed);
    }
}

struct RustMiscdev {
    _dev: Pin<Box<miscdev::Registration<Producer>>>,
}

impl kernel::Module for RustMiscdev {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        // pr_info!("waiting for gdb for 5 seconds\n");
        // delay::coarse_sleep(Duration::from_secs(5));
        pr_info!("atomic mmap (init)\n");
        let state = Producer::try_new()?;
        Ok(RustMiscdev {
            _dev: miscdev::Registration::new_pinned(fmt!("{name}"), state)?,
        })
    }
}

impl Drop for RustMiscdev {
    fn drop(&mut self) {
        pr_info!("Rust miscellaneous device sample (exit)\n");
        // delay::coarse_sleep(Duration::from_secs(1));
    }
}
