use crate::timestamp::TimestampRecorder;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::time::SystemTime;
use std::{env, fs};

mod prog_mmap;
mod prog_mmap_cstmsg;
mod prog_mmap_nosync;
mod prog_read;
mod prog_read_cstmsg;
mod timestamp;
mod utils;

const DEFAULT_FILE_PATH: &str = "/dev/rustdev";
const DEFAULT_DATA_OFFSET: usize = 4096;
const DEFAULT_MSG_SLOTS: usize = 8;
const DEFAULT_MSG_SIZE: usize = 4096;
const DEFAULT_MSG_NUM: usize = 1000;

pub struct Config {
    file_path: String,
    data_offset: usize,
    msg_slots: usize,
    msg_size: usize,
    msg_num: usize,
}

impl Config {
    fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();
        let file_path = match args.next() {
            Some(arg) => arg,
            None => DEFAULT_FILE_PATH.to_string(),
        };
        let data_offset = match args.next() {
            Some(arg) => arg.parse().unwrap(),
            None => DEFAULT_DATA_OFFSET,
        };
        let msg_slots = match args.next() {
            Some(arg) => arg.parse().unwrap(),
            None => DEFAULT_MSG_SLOTS,
        };
        let msg_size = match args.next() {
            Some(arg) => arg.parse().unwrap(),
            None => DEFAULT_MSG_SIZE,
        };
        let msg_num = match args.next() {
            Some(arg) => arg.parse().unwrap(),
            None => DEFAULT_MSG_NUM,
        };
        Ok(Config {
            file_path,
            data_offset,
            msg_slots,
            msg_size,
            msg_num,
        })
    }
}

impl ToString for Config {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s += "file_path: ";
        s += &self.file_path;
        s += "\ndata_offset: ";
        s += &self.data_offset.to_string();
        s += "\nmsg_slots: ";
        s += &self.msg_slots.to_string();
        s += "\nmsg_size: ";
        s += &self.msg_size.to_string();
        s += "\nmsg_num: ";
        s += &self.msg_num.to_string();
        s
    }
}

struct Metrics {
    data_size: usize,
    sync: bool,
    recorder: TimestampRecorder,
}

impl Metrics {
    fn store_info(&self) {
        let siz_b = self.data_size;

        let mut s = String::new();
        write!(&mut s, "{} {}\n", siz_b, self.sync).unwrap();
        self.recorder
            .iter()
            .for_each(|ts| write!(&mut s, "{}\n", ts).unwrap());

        let mut f = fs::File::options()
            .create(true)
            .append(true)
            .open("./log")
            .unwrap();
        write!(&mut f, "{s}").unwrap();
    }
}

fn main() {
    let config = Config::build(env::args()).unwrap();
    let res = prog_mmap_cstmsg::run(config);
    res.store_info();
}
