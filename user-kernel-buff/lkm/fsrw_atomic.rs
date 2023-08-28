// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

mod message;
mod timestamp;
mod vallocator;
use crate::message::Message;
use crate::timestamp::*;
use crate::vallocator::VAllocator;
use alloc::vec::Vec;
use core::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
    u8,
};
use kernel::prelude::*;
use kernel::{
    bindings, delay,
    file::{File, IoctlCommand, IoctlHandler, Operations},
    io_buffer::IoBufferWriter,
    miscdev,
    sync::{Arc, ArcBorrow, UniqueArc},
    task::Task,
    user_ptr::{UserSlicePtr, UserSlicePtrWriter},
};
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
    ticker: AtomicUsize,
    head: AtomicUsize,
    tail: AtomicUsize,
    data: Vec<u8, VAllocator>,
    recorder_r: Vec<Timestamp, VAllocator>,
    recorder_w: Vec<Timestamp, VAllocator>,
    ts_trans_rcnt: AtomicUsize,
    ts_trans_wcnt: AtomicUsize,
}

impl Producer {
    fn try_new() -> Result<Arc<Producer>> {
        let len = *msg_size.read() * *msg_slots.read();
        let pinned = Pin::from(UniqueArc::try_new(Self {
            stop: AtomicBool::new(false),
            ticker: AtomicUsize::new(0),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            data: {
                let mut vec = Vec::try_with_capacity_in(len, VAllocator)?;
                vec.try_resize(len, 0)?;
                vec
            },
            recorder_r: Vec::new_in(VAllocator),
            recorder_w: Vec::new_in(VAllocator),
            ts_trans_rcnt: AtomicUsize::new(0),
            ts_trans_wcnt: AtomicUsize::new(0),
        })?);
        Ok(pinned.into())
    }

    fn reset(&self) {
        self.stop.store(false, Ordering::SeqCst);
        self.head.store(0, Ordering::SeqCst);
        self.tail.store(0, Ordering::SeqCst);
        self.ticker.store(0, Ordering::SeqCst);

        let this = unsafe { &mut *(self as *const Self as *mut Self) };
        this.recorder_r.clear();
        this.recorder_w.clear();
    }
}

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
                let lenr: u32 = this.recorder_r.len() as _;
                let lenw: u32 = this.recorder_w.len() as _;
                this.ts_trans_rcnt.store(lenr as _, Ordering::SeqCst);
                this.ts_trans_wcnt.store(lenw as _, Ordering::SeqCst);
                
                let len = lenr + lenw;
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
                let lenr = this.ts_trans_rcnt.load(Ordering::SeqCst);
                let lenw = this.ts_trans_wcnt.load(Ordering::SeqCst);

                let rcder_r = &this.recorder_r[0..lenr];
                rcder_r.iter().for_each(|x| {
                    raw_vec.try_extend_from_slice(x.into_raw().as_slice()).unwrap()
                });

                let rcder_w = &this.recorder_w[0..lenw];
                rcder_w.iter().for_each(|x| {
                    raw_vec.try_extend_from_slice(x.into_raw().as_slice()).unwrap()
                });
                
                let user_ptr = unsafe { UserSlicePtr::new(arg as _, raw_vec.len()) };
                let mut writer = user_ptr.writer();
                pr_info!("{} - {}\n", writer.len(), raw_vec.len());
                writer.write_slice(raw_vec.as_slice()).unwrap();
                Ok(0)
            }
            _ => panic!("unknown command"),
        }
    }
}

#[vtable]
impl Operations for Producer {
    type Data = Arc<Producer>;
    type OpenData = Arc<Producer>;

    fn open(this: &Self::OpenData, _file: &File) -> Result<Self::Data> {
        let shared = this.clone();
        shared.reset();
        let mut ticker: u64 = 0;
        let msgsiz = *msg_size.read();
        let _ = Task::spawn(fmt!("producer"), move || loop {
            let msg = Message::from_msgid_with_size(ticker, msgsiz);
            let content = msg.as_bytes();

            let head;
            let mut tail;
            let next_head;

            // timestamp: start sync for write
            let that = unsafe { &mut *(&shared.recorder_w as *const Vec<Timestamp, VAllocator>).cast_mut() };
            that.try_push(Timestamp::new(TimestampTag::WriteSyncStart, ticker)).unwrap();
            {
                head = shared.head.load(Ordering::SeqCst);
                tail = shared.tail.load(Ordering::SeqCst);
                next_head = (head + 1) % *msg_slots.read();
                while next_head == tail {
                    if shared.stop.load(Ordering::SeqCst) {
                        pr_info!("exit normally\n");
                        return;
                    }
                    tail = shared.tail.load(Ordering::SeqCst);
                }
            }
            // timestamp: end sync for write
            that.try_push(Timestamp::new(TimestampTag::WriteSyncEnd, ticker)).unwrap();

            let siz = *msg_size.read();
            let start = siz * head;
            let end = start + msgsiz; // implicity: msg_size == msg.len
            let slot = &shared.data[start..end] as *const [u8] as *mut [u8];
            let slot = unsafe { &mut *slot };

            // timestamp: start write
            that.try_push(Timestamp::new(TimestampTag::WriteStart, ticker)).unwrap();
            {
                slot.copy_from_slice(content);
            }
            // timestamp: end write
            that.try_push(Timestamp::new(TimestampTag::WriteEnd, ticker)).unwrap();

            shared.head.store(next_head, Ordering::SeqCst);
            ticker += 1;
        });
        Ok(this.clone())
    }

    fn release(this: Self::Data, _file: &File) {
        this.stop.store(true, Ordering::SeqCst);
    }

    fn ioctl(data: ArcBorrow<'_, Producer>, file: &File, cmd: &mut IoctlCommand) -> Result<i32> {
        let _ = cmd.dispatch::<Self>(&data, file);
        Ok(1)
    }

    
    fn read(
        this: ArcBorrow<'_, Producer>,
        _file: &File,
        writer: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        let ticker: u64 = this.ticker.fetch_add(1, Ordering::SeqCst) as _;
        let tail;
        let next_tail;

        // timestamp: start sync for read
        let that = unsafe { &mut *(&this.recorder_r as *const Vec<Timestamp, VAllocator>).cast_mut() };
        that.try_push(Timestamp::new(TimestampTag::ReadSyncStart, ticker)).unwrap();
        {
            let mut head = this.head.load(Ordering::SeqCst);
            tail = this.tail.load(Ordering::SeqCst);
            next_tail = (tail + 1) % *msg_slots.read();
            while head == tail {
                head = this.head.load(Ordering::SeqCst);
            }
        }
        // timestamp: end sync for read
        that.try_push(Timestamp::new(TimestampTag::ReadSyncEnd, ticker)).unwrap();

        let siz = *msg_size.read();
        let start = siz * tail;
        let end = start + siz;
        let msg = &this.data[start..end] as *const [u8] as *mut [u8];
        let msg = unsafe { &mut *msg };

        // timestamp: start read
        that.try_push(Timestamp::new(TimestampTag::ReadStart, ticker)).unwrap();
        {
            writer.write_slice(msg).unwrap();
        }
        // timestamp: end read
        that.try_push(Timestamp::new(TimestampTag::ReadEnd, ticker)).unwrap();

        this.tail.store(next_tail, Ordering::SeqCst);
        Ok(siz)
    }
}

struct RustMiscdev {
    _dev: Pin<Box<miscdev::Registration<Producer>>>,
}

impl kernel::Module for RustMiscdev {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        // pr_info!("waiting for gdb for 5 seconds\n");
        // delay::coarse_sleep(Duration::from_secs(5));
        pr_info!("Rust atomic fs read/write (init)\n");
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
