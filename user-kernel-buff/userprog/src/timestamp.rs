use std::fmt::{Display, Formatter, Write as _};
use std::ops::{Deref, DerefMut};

use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum TimestampTag {
    ReadSyncStart,
    ReadSyncEnd,
    ReadStart,
    ReadEnd,
    WriteSyncStart,
    WriteSyncEnd,
    WriteStart,
    WriteEnd,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct Timestamp {
    tag: TimestampTag,
    ku: bool,
    // true => kernel, false => userspace
    ticker: u64,
    // which msg
    time: u64, // timestamp in ns
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {} {}",
            self.tag,
            if self.ku { "kernel" } else { "user" },
            self.ticker,
            self.time
        )
    }
}

impl Timestamp {
    fn get_time_ns() -> u64 {
        let ns = unsafe {
            let mut tp = timespec {
                tv_sec: 0 as _,
                tv_nsec: 0 as _,
            };
            clock_gettime(CLOCK_MONOTONIC, &mut tp as *mut timespec);
            tp.tv_sec * 1_000_000_000 + tp.tv_nsec
        };
        ns as _
    }

    pub(crate) fn new(tag: TimestampTag, ticker: u64) -> Timestamp {
        Self {
            tag,
            ku: false,
            ticker,
            time: Self::get_time_ns(),
        }
    }

    pub(crate) fn generate_bytes(&self) -> [u8; 18] {
        let mut res = [0u8; 18];
        res[0] = self.tag as u8;
        res[1] = self.ku.into();
        res[2..2 + 8].copy_from_slice(self.ticker.to_ne_bytes().as_slice());
        res[2 + 8..2 + 8 + 8].copy_from_slice(self.time.to_ne_bytes().as_slice());
        res
    }

    pub(crate) fn from_bytes(raw: &[u8]) -> Timestamp {
        let tag = match raw[0] {
            0 => TimestampTag::ReadSyncStart,
            1 => TimestampTag::ReadSyncEnd,
            2 => TimestampTag::ReadStart,
            3 => TimestampTag::ReadEnd,
            4 => TimestampTag::WriteSyncStart,
            5 => TimestampTag::WriteSyncEnd,
            6 => TimestampTag::WriteStart,
            7 => TimestampTag::WriteEnd,
            _ => panic!(),
        };
        let ku = raw[1] != 0;
        let ticker = {
            let mut ticker_bytes = [0u8; 8];
            ticker_bytes.copy_from_slice(&raw[2..2 + 8]);
            u64::from_ne_bytes(ticker_bytes)
        };
        let time = {
            let mut time_bytes = [0u8; 8];
            time_bytes.copy_from_slice(&raw[2 + 8..2 + 8 + 8]);
            u64::from_ne_bytes(time_bytes)
        };
        Timestamp {
            tag,
            ku,
            ticker,
            time,
        }
    }
}

pub(crate) struct TimestampRecorder {
    records: Vec<Timestamp>,
}

impl Deref for TimestampRecorder {
    type Target = Vec<Timestamp>;

    fn deref(&self) -> &Self::Target {
        &self.records
    }
}

impl DerefMut for TimestampRecorder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.records
    }
}

impl TimestampRecorder {
    pub(crate) fn new() -> TimestampRecorder {
        let records = Vec::<Timestamp>::new();
        TimestampRecorder { records }
    }

    pub(crate) fn analyze(&self) -> (u64, u64) {
        todo!()
    }
}
