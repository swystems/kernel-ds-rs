use std::ffi::CStr;
use std::fs::File;
use std::io::Read;
use std::os::fd::AsRawFd;

use crate::timestamp::{Timestamp, TimestampRecorder, TimestampTag};
use crate::utils::get_kern_timestamps;
use crate::{Config, Metrics};

pub(crate) fn run(config: Config) -> Metrics {
    let mut recorder = TimestampRecorder::new();

    let mut file = File::open(&config.file_path).expect("cannot open file");
    let mut sync = true;
    let mut msg = Vec::with_capacity(config.msg_size);
    unsafe {
        msg.set_len(config.msg_size);
    }
    let mut data_size: usize = 0;

    for msg_cnt in 0..config.msg_num {
        // timestamp: read start
        recorder.push(Timestamp::new(TimestampTag::ReadStart, msg_cnt as _));
        {
            data_size += file.read(&mut msg[..]).expect("failed to read");
        }
        // timestamp: read end
        recorder.push(Timestamp::new(TimestampTag::ReadEnd, msg_cnt as _));

        let s = CStr::from_bytes_until_nul(&msg)
            .expect("cannot convert &[u8] into CStr")
            .to_str()
            .expect("cannot convert CStr into &str");
        let msg_id: usize = s.trim().parse().expect("failed to parse msg_id");

        if msg_id != msg_cnt {
            sync = false;
            break;
        }
    }

    let rcder_k = get_kern_timestamps(file.as_raw_fd());
    recorder.extend(rcder_k.iter());

    Metrics {
        data_size,
        sync,
        recorder,
    }
}
