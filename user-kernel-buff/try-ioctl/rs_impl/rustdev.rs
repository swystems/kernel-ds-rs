// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

use core::time::Duration;
use core::u8;
use kernel::prelude::*;
use kernel::{
    delay,
    file::{File, IoctlCommand, IoctlHandler, Operations},
    io_buffer::IoBufferWriter,
    miscdev,
    str::CString,
    sync::{Arc, ArcBorrow, CondVar, Mutex, UniqueArc},
    task::Task,
    user_ptr::UserSlicePtrWriter,
};
module! {
    type: RustMiscdev,
    name: "rustdev",
    author: "Rust for Linux Contributors",
    description: "Rust miscellaneous device sample",
    license: "GPL",
}
const MAX_MSGS: usize = 10;
const MSG_SIZE: usize = 128;

struct Producer {
    stop: bool,
    ticker: usize,
    head: usize,
    tail: usize,
    queue: [u8; MSG_SIZE * MAX_MSGS],
    // the times of is_full==false minus the times of is_empty==true
    // roughly indicates whether the consumer or the producer is faster
    metrics: isize, 
}

impl Producer {
    fn new() -> Producer {
        Producer {
            stop: false,
            ticker: 0,
            head: 0,
            tail: 0,
            queue: [0; MSG_SIZE * MAX_MSGS],
            metrics: 0,
        }
    }
    fn is_empty(&mut self) -> bool {
        if self.head == self.tail {
            self.metrics -= 1;
            true
        } else {
            false
        }
    }
    fn is_full(&mut self) -> bool {
        if self.head == (self.tail + 1) % MAX_MSGS {
            self.metrics += 1;
            true
        } else {
            false
        }
    }

    // read from mapped page and write to buffer
    fn write_to(&mut self, writer: &mut UserSlicePtrWriter) {
        let start = MSG_SIZE * self.tail;
        let end = start + MSG_SIZE;

        let msg = &self.queue[start..end];
        if let Err(_) = writer.write_slice(msg) {
            pr_info!(
                "msg.len() = {} | userbuffer.len() = {}\n",
                msg.len(),
                writer.len()
            );
        }
        self.tail = (self.tail + 1) % MAX_MSGS;
    }

    // write to mapped pages
    fn produce(&mut self) {
        // delay::coarse_sleep(Duration::from_millis(2));
        let msg = CString::try_from_fmt(fmt!("message {}", self.ticker)).unwrap();
        let len = msg.len_with_nul();
        let start = MSG_SIZE * self.head;
        let end = start + len;
        self.queue[start..end].copy_from_slice(msg.as_bytes_with_nul());
        self.head = (self.head + 1) % MAX_MSGS;
        self.ticker += 1;
    }
}

struct FileState {
    state_changed: CondVar,
    inner: Mutex<Producer>,
}
unsafe impl Send for FileState {}
unsafe impl Sync for FileState {}
impl FileState {
    fn try_new() -> Result<Arc<Self>> {
        let mut state = Pin::from(UniqueArc::try_new(Self {
            state_changed: unsafe { CondVar::new() },
            inner: unsafe { Mutex::new(Producer::new()) },
        })?);
        let pinned = unsafe { state.as_mut().map_unchecked_mut(|s| &mut s.state_changed) };
        kernel::condvar_init!(pinned, "SharedState::state_changed");
        let pinned = unsafe { state.as_mut().map_unchecked_mut(|s| &mut s.inner) };
        kernel::mutex_init!(pinned, "SharedState::inner");

        Ok(state.into())
    }
}

impl IoctlHandler for FileState {
    type Target<'a> = &'a Self;

    fn read(this: &Self, _file: &File, _cmd: u32, writer: &mut UserSlicePtrWriter) -> Result<i32> {
        {
            let mut inner = this.inner.lock();
            while inner.is_empty() {
                if inner.stop || this.state_changed.wait(&mut inner) {
                    return Err(EINVAL);
                }
            }
            inner.write_to(writer);
        } // have to unlock before notify_all
        this.state_changed.notify_all();
        Ok(1)
    }
}

#[vtable]
impl Operations for FileState {
    type Data = Arc<FileState>;
    type OpenData = Arc<FileState>;

    fn open(shared: &Self::OpenData, _file: &File) -> Result<Self::Data> {
        let state = shared.clone();

        // producer
        let _ = Task::spawn(fmt!("producer"), move || loop {
            {
                let mut inner = state.inner.lock();
                while inner.is_full() {
                    if inner.stop {
                        pr_info!("exit normally\n");
                        return;
                    } else if state.state_changed.wait(&mut inner) {
                        pr_info!("[Error]: exit abnormal \n");
                        return;
                    }
                }
                inner.produce();
            }
            state.state_changed.notify_all();
        });

        Ok(shared.clone())
    }

    fn release(data: Self::Data, _file: &File) {
        let mut inner = data.inner.lock();
        inner.stop = true;
        pr_info!("metrics = {}\n", inner.metrics);
    }

    fn ioctl(data: ArcBorrow<'_, FileState>, file: &File, cmd: &mut IoctlCommand) -> Result<i32> {
        let _ = cmd.dispatch::<Self>(&data, file);
        Ok(1)
    }
}

struct RustMiscdev {
    _dev: Pin<Box<miscdev::Registration<FileState>>>,
}

impl kernel::Module for RustMiscdev {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        delay::coarse_sleep(Duration::from_secs(1));
        pr_info!("Rust miscellaneous device sample (init)\n");
        let state = FileState::try_new()?;
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
