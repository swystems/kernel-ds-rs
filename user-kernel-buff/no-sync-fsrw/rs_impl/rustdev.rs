// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

// use core::sync::atomic::{fence, Ordering};
use alloc::vec::Vec;
use core::{
    time::Duration,
    u8,
    ops::Deref,
    cell::UnsafeCell
};
use kernel::prelude::*;
use kernel::{
    delay,
    file::{File, Operations},
    io_buffer::IoBufferWriter,
    miscdev,
    str::CString,
    sync::{Arc, ArcBorrow, UniqueArc},
    task::Task,
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
            default: 80,
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

struct SyncUnsafeCell<T>(UnsafeCell<T>);
impl<T> SyncUnsafeCell<T> {
    fn new(inner: T) -> SyncUnsafeCell<T> {
        SyncUnsafeCell(UnsafeCell::new(inner))
    }

    fn as_mut(&self) -> &mut T {
        unsafe {&mut *self.0.get()}
    }
}
impl<T> Deref for SyncUnsafeCell<T> {
    type Target = UnsafeCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
unsafe impl<T> Sync for SyncUnsafeCell<T> {}

struct ProducerInner {
    stop: bool,
    head: usize,
    tail: usize,
    data: Vec<u8>,
}
struct Producer {
    inner: SyncUnsafeCell<ProducerInner>
}
impl Deref for Producer {
    type Target = SyncUnsafeCell<ProducerInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Producer {
    fn try_new() -> Result<Arc<Producer>> {
        let len = *msg_size.read() * *msg_slots.read();

        let inner = ProducerInner {
            stop: false,
            head: 0,
            tail: 0,
            data: {
                let mut vec = Vec::try_with_capacity(len).unwrap();
                vec.try_resize(len, 0)?;
                vec
            },
        };

        let pinned = Pin::from(UniqueArc::try_new(
            Producer { inner: SyncUnsafeCell::new(inner) }
        )?);
        Ok(pinned.into())
    }

    fn reset(&self) {
        let inner = self.inner.as_mut();
        inner.stop = false;
        inner.head = 0;
        inner.tail = 0;
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
        let slots = *msg_slots.read();
        let siz = *msg_size.read();
        let _ = Task::spawn(fmt!("producer"), move || loop {
            {
                let msg = CString::try_from_fmt(fmt!("message {ticker}")).unwrap();
                let msg = msg.as_bytes_with_nul();
                ticker += 1;

                let inner = shared.inner.as_mut();
                let head = inner.head;
                let next_head = (head + 1) % slots;
                while next_head == inner.tail {
                    if inner.stop {
                        pr_info!("exit normally\n");
                        return;
                    }
                    // fence(Ordering::SeqCst);
                }

                let start = siz * head;
                let end = start + msg.len();
                let slot = &mut inner.data[start..end];
                slot.copy_from_slice(msg);
                // fence(Ordering::SeqCst);
                inner.head = next_head;
            }
        });
        Ok(this.clone())
    }

    fn release(data: Self::Data, _file: &File) {
        data.inner.as_mut().stop = true;
    }

    fn read(
        data: ArcBorrow<'_, Producer>,
        _file: &File,
        writer: &mut impl IoBufferWriter,
        _offset: u64
    ) -> Result<usize> {
        let inner = data.inner.as_mut();
        let tail = inner.tail;
        let next_tail = (tail + 1) % *msg_slots.read();

        while inner.head == tail {
            // fence(Ordering::SeqCst);
        }

        let siz = *msg_size.read();
        let start = siz * tail;
        let end = start + siz;
        let msg = &inner.data[start..end];
        writer.write_slice(msg).unwrap();
        // fence(Ordering::SeqCst);
        inner.tail = next_tail;
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
        delay::coarse_sleep(Duration::from_secs(1));
    }
}
