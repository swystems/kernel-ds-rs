// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

use core::u8;
use kernel::prelude::*;
use kernel::{
    // delay,
    file::{File, Operations},
    io_buffer::IoBufferWriter,
    miscdev,
    mm::virt::Area,
    pages::Pages,
    str::CString,
    sync::{Arc, ArcBorrow, CondVar, Mutex, UniqueArc},
    task::Task,
};
module! {
    type: RustMiscdev,
    name: "rust_miscdev",
    author: "Rust for Linux Contributors",
    description: "Rust miscellaneous device sample",
    license: "GPL",
}
const MAX_MSGS: usize = 3;
const MSG_SIZE: usize = 32;

struct Producer {
    stop: bool,
    msg_cnt: usize,
    ticker: usize,
    pages: Pages<0>,
}

impl Producer {
    // notice!!!:
    // pages only will be mapped when reading and writing
    // they will be unmapped at the end of pages.read() / pages.write()
    // see source code
    // https://rust-for-linux.github.io/docs/rust/src/kernel/pages.rs.html#70-80
    fn new() -> Producer {
        Producer {
            stop: false,
            msg_cnt: 0,
            ticker: 0,
            pages: Pages::<0>::new().unwrap(),
        }
    }

    // read from mapped page and write to buffer
    fn write_to(&mut self, data: &mut impl IoBufferWriter) {
        // pr_info!("reading...\n");
        // delay::coarse_sleep(Duration::from_secs(1));
        self.msg_cnt -= 1;

        // todo: need optimization - transfer data without `bridge`
        let start = MSG_SIZE * self.msg_cnt;
        let bridge = &mut [0u8; MSG_SIZE];
        unsafe {
            self.pages
                .read(bridge.as_mut_ptr(), start, MSG_SIZE)
                .unwrap();
        }
        let _ = data.write_slice(bridge);

        // pr_info!("read!\n");
    }

    // write to mapped pages
    fn produce(&mut self) {
        // pr_info!("writing...\n");
        let msg = CString::try_from_fmt(fmt!("message {}:{}", self.ticker, self.msg_cnt)).unwrap();
        let len = msg.len_with_nul();
        self.ticker += 1;
        // pr_info!("{}\n", msg.to_str().unwrap());
        unsafe {
            self.pages
                .write(msg.as_ptr(), MSG_SIZE * self.msg_cnt, len)
                .unwrap();
        }
        self.msg_cnt += 1;
        // pr_info!("wrote!\n");
    }
}

struct SharedState {
    state_changed: CondVar,
    inner: Mutex<Producer>,
}
unsafe impl Send for SharedState {}
unsafe impl Sync for SharedState {}
impl SharedState {
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

struct Message;
#[vtable]
impl Operations for Message {
    type Data = Arc<SharedState>;
    type OpenData = Arc<SharedState>;

    fn open(shared: &Arc<SharedState>, _file: &File) -> Result<Self::Data> {
        let state = shared.clone();
        // producer
        let _ = Task::spawn(fmt!("producer"), move || {
            while !state.inner.lock().stop {
                {
                    let mut inner = state.inner.lock();
                    while inner.msg_cnt == MAX_MSGS && !inner.stop {
                        if state.state_changed.wait(&mut inner) {
                            return;
                        }
                    }
                    inner.produce();
                }
                state.state_changed.notify_all();
            }
            // pr_info!("stopped\n");
        });
        Ok(shared.clone())
    }

    fn release(data: Self::Data, _file: &File) {
        data.inner.lock().stop = true;
    }

    fn mmap(data: ArcBorrow<'_, SharedState>, _file: &File, vma: &mut Area) -> Result {
        let inner = data.inner.lock();
        let _ = vma.insert_page(vma.start(), &inner.pages);
        Ok(())
    }

    fn read(
        shared: ArcBorrow<'_, SharedState>,
        _: &File,
        data: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        {
            let mut inner = shared.inner.lock();
            while inner.msg_cnt == 0 {
                if shared.state_changed.wait(&mut inner) {
                    return Err(EINTR);
                }
            }
            // todo: need optimization
            inner.write_to(data);
        }
        shared.state_changed.notify_all();

        Ok(1)
    }
}

struct RustMiscdev {
    _dev: Pin<Box<miscdev::Registration<Message>>>,
}

impl kernel::Module for RustMiscdev {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust miscellaneous device sample (init)\n");
        let state = SharedState::try_new()?;
        Ok(RustMiscdev {
            _dev: miscdev::Registration::new_pinned(fmt!("{name}"), state)?,
        })
    }
}

impl Drop for RustMiscdev {
    fn drop(&mut self) {
        pr_info!("Rust miscellaneous device sample (exit)\n");
    }
}
