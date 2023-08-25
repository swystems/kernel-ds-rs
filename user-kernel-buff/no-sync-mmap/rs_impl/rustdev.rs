// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

use alloc::vec::Vec;
use core::{
    mem,
    sync::atomic::{
        fence, 
        AtomicBool, Ordering
    },
    time::Duration,
    u8,
};
use kernel::prelude::*;
use kernel::{
    delay,
    file::{File, Operations},
    miscdev,
    mm::virt::Area,
    pages::Pages,
    str::CString,
    sync::{Arc, ArcBorrow, UniqueArc},
    task::Task,
    PAGE_SIZE,
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
    head_tail: Pages<0>,
    queue: Vec<Pages<0>>,
}
unsafe impl Send for Producer {}
unsafe impl Sync for Producer {}

impl Producer {
    fn try_new() -> Result<Arc<Producer>> {
        let slots = *msg_slots.read();
        let pinned = Pin::from(UniqueArc::try_new(Self {
            stop: AtomicBool::new(false),
            head_tail: Pages::<0>::new().unwrap(),
            queue: {
                let mut vec = Vec::try_with_capacity(slots).unwrap();
                for _ in 0..slots {
                    let page = Pages::<0>::new().unwrap();
                    vec.try_push(page).unwrap();
                }
                vec
            },
        })?);
        Ok(pinned.into())
    }

    fn reset(&self) {
        self.stop.store(false, Ordering::SeqCst);
        let hd_tl = [0usize; 2];
        unsafe {
            self.head_tail
                .write(hd_tl.as_ptr() as *const u8, 0, mem::size_of::<[usize; 2]>())
                .unwrap()
        };
    }
}

#[vtable]
impl Operations for Producer {
    type Data = Arc<Producer>;
    type OpenData = Arc<Producer>;

    fn open(this: &Self::OpenData, _file: &File) -> Result<Self::Data> {
        let shared = this.clone();
        shared.reset();
        let mut ticker = 0;
        let _ = Task::spawn(fmt!("producer"), move || loop {
            {
                let slots = *msg_slots.read();

                // produce message
                let msg = CString::try_from_fmt(fmt!("message {}", ticker)).unwrap();
                let len = msg.len_with_nul();
                ticker += 1;

                // get head & tail
                let mut head_tail = [0usize; 2];
                unsafe {
                    shared.head_tail
                        .read(
                            head_tail.as_mut_ptr() as *mut u8,
                            0,
                            mem::size_of::<[usize; 2]>(),
                        )
                        .unwrap();
                }
                let next_head = (head_tail[0] + 1) % slots;

                // it is the slot which we will write into
                let page = &shared.queue[head_tail[0] as usize];

                // spin until user increase tail
                while next_head == head_tail[1] {
                    if shared.stop.load(Ordering::Relaxed) {
                        pr_info!("exit normally\n");
                        return;
                    }

                    // get head & tail
                    unsafe {
                        shared.head_tail
                            .read(
                                head_tail[1..=1].as_mut_ptr() as *mut u8,
                                mem::size_of::<usize>(),
                                mem::size_of::<usize>(),
                            )
                            .unwrap()
                    }
                    // fence(Ordering::SeqCst);
                }

                // write message
                unsafe {
                    page.write(msg.as_char_ptr() as _, 0, len).unwrap();
                }
                // pr_info!("{}\n", msg.to_str().unwrap());

                // make sure `write` will be finished before increase head
                // fence(Ordering::SeqCst);

                // increase head
                let head = (head_tail[0] + 1) % slots;
                let head_bytes = head.to_ne_bytes();
                unsafe {
                    shared.head_tail
                        .write(head_bytes.as_slice().as_ptr(), 0, head_bytes.len())
                        .unwrap();
                }
            }
        });
        Ok(this.clone())
    }

    fn mmap(data: ArcBorrow<'_, Producer>, _file: &File, vma: &mut Area) -> Result {
        vma.insert_page(vma.start(), &data.head_tail)
            .unwrap_or_else(|_| pr_info!("failed to mapping head_tail page\n"));
        for i in 1..=*msg_slots.read() {
            let offset = PAGE_SIZE * i;
            vma.insert_page(vma.start() + offset, &data.queue[i - 1])
                .unwrap_or_else(|_| pr_info!("first failed at buffer page[{i}]\n"));
        }
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
        pr_info!("mmap nosync (init)\n");
        let state = Producer::try_new()?;
        Ok(RustMiscdev {
            _dev: miscdev::Registration::new_pinned(fmt!("{name}"), state)?,
        })
    }
}

impl Drop for RustMiscdev {
    fn drop(&mut self) {
        pr_info!("mmap nosync (exit)\n");
        delay::coarse_sleep(Duration::from_secs(1));
    }
}
