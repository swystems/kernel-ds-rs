use std::ffi::CStr;
use std::fs::File;
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicUsize, Ordering};

use libc::{mmap64, MAP_SHARED, PROT_READ, PROT_WRITE};

use crate::timestamp::{Timestamp, TimestampRecorder, TimestampTag};
use crate::utils::get_kern_timestamps;
use crate::{Config, Metrics};

struct Buffer {
    head: *const AtomicUsize,
    tail: *const AtomicUsize,
    data: *const u8,
    config: Config,
}

impl Buffer {
    fn read(&self, ticker: u64, recv: &mut [u8], rcder: &mut TimestampRecorder) {
        let tail;
        let next_tail;

        // -------- timestamp: start sync for read --------
        rcder.push(Timestamp::new(TimestampTag::ReadSyncStart, ticker));
        {
            let head_ref = unsafe { self.head.as_ref() }.expect("head pointer is null");
            let mut head = head_ref.load(Ordering::SeqCst);
            let tail_ref = unsafe { self.tail.as_ref() }.expect("tail pointer is null");
            tail = tail_ref.load(Ordering::SeqCst);
            next_tail = (tail + 1) % self.config.msg_slots;

            while head == tail {
                head = head_ref.load(Ordering::SeqCst);
            }
        }
        // -------- timestamp: end sync for read --------
        rcder.push(Timestamp::new(TimestampTag::ReadSyncEnd, ticker));

        // -------- timestamp: start read --------
        rcder.push(Timestamp::new(TimestampTag::ReadStart, ticker));
        {
            let offset = self.config.msg_size * tail;
            unsafe {
                self.data
                    .add(offset)
                    .copy_to_nonoverlapping(recv.as_mut_ptr(), self.config.msg_size);
            }
        }
        // -------- timestamp: end read --------
        rcder.push(Timestamp::new(TimestampTag::ReadEnd, ticker));

        unsafe {
            (*self.tail).store(next_tail, Ordering::SeqCst);
        }
    }
}

pub(crate) fn run(config: Config) -> Metrics {
    let f = File::options()
        .read(true)
        .write(true)
        .open(&config.file_path)
        .expect("cannot open file");

    // create buffer
    let bufsiz = config.data_offset + config.msg_size * config.msg_slots;

    let mp = unsafe {
        mmap64(
            0 as _,
            bufsiz,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            f.as_raw_fd(),
            0,
        )
    };

    let head: *const AtomicUsize = mp as *mut usize as _;
    let tail = unsafe { head.add(1) };
    let data = unsafe { mp.add(config.data_offset) } as _;
    let buffer = Buffer {
        head,
        tail,
        data,
        config,
    };

    let sync = true;
    let mut data_size: usize = 0;

    let mut msg = Vec::with_capacity(buffer.config.msg_size);
    unsafe {
        msg.set_len(buffer.config.msg_size);
    }

    let mut recorder = TimestampRecorder::new();

    for msg_cnt in 0..buffer.config.msg_num {
        buffer.read(msg_cnt as _, &mut msg[..], &mut recorder);
        data_size += msg.len();
    }

    let rcder_k = get_kern_timestamps(f.as_raw_fd());
    recorder.extend(rcder_k.iter());

    Metrics {
        data_size,
        sync,
        recorder,
    }
}
