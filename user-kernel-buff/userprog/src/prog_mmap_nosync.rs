use std::ffi::CStr;
use std::fs::File;
use std::os::fd::AsRawFd;

use libc::{mmap64, MAP_SHARED, PROT_READ, PROT_WRITE};

use crate::Config;

struct Buffer {
    head: *mut usize,
    tail: *mut usize,
    data: *const u8,
    config: Config,
}

impl Buffer {
    fn read(&self, recv: &mut [u8]) {
        let head_ref = unsafe { self.head.as_mut() }.expect("head pointer is null");
        let tail_ref = unsafe { self.tail.as_mut() }.expect("tail pointer is null");
        let tail = *tail_ref;

        let next_tail = (tail + 1) % self.config.msg_slots;
        while *head_ref == tail {}

        let offset = self.config.msg_size * tail;
        unsafe {
            self.data
                .add(offset)
                .copy_to_nonoverlapping(recv.as_mut_ptr(), self.config.msg_size);
        }
        *tail_ref = next_tail;
    }
}

pub(crate) fn run(config: Config) -> (usize, bool) {
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

    let head = mp as *mut usize;
    let tail = unsafe { head.add(1) };
    let data = unsafe { mp.add(config.data_offset) } as _;
    let buffer = Buffer {
        head,
        tail,
        data,
        config,
    };

    let mut sync = true;
    let mut sum_siz: usize = 0;

    let mut msg = Vec::with_capacity(buffer.config.msg_size);
    unsafe {
        msg.set_len(buffer.config.msg_size);
    }

    for msg_cnt in 0..buffer.config.msg_num {
        buffer.read(&mut msg[..]);
        sum_siz += msg.len();
        let msg =
            CStr::from_bytes_until_nul(&msg[..]).expect("cannot convert message into CString");
        let msg_id: usize = msg
            .to_str()
            .expect("cannot convert CString into &str")
            .strip_prefix("message ")
            .expect("wrong message format. It should be `message <msg_id>`")
            .parse()
            .expect("failed to parse msg_id");

        if msg_id != msg_cnt {
            sync = false;
            break;
        }
    }

    (sum_siz, sync)
}
