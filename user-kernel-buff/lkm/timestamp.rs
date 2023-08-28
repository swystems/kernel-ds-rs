// use crate::vallocator::VAllocator;
// use core::ops::{Deref, DerefMut};
use kernel::bindings::ktime_get;
// use kernel::prelude::*;

// todo: try to refactor with macro

#[repr(u8)]
#[derive(Copy, Clone)]
#[derive(PartialEq)]
#[allow(dead_code)]
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

#[allow(dead_code)]
pub(crate) struct Timestamp {
    tag: TimestampTag,
    ku: bool,
    ticker: u64,
    pub(crate) time: u64,
}

impl Timestamp {
    #[allow(dead_code)]
    pub(crate) fn new(tag: TimestampTag, ticker: u64) -> Timestamp {
        Self {
            tag,
            ku: true,
            ticker,
            time: unsafe { ktime_get() } as _,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn into_raw(&self) -> [u8; 18] {
        let mut res = [0u8; 18];
        res[0] = match self.tag {
            TimestampTag::ReadSyncStart => 0,
            TimestampTag::ReadSyncEnd => 1,
            TimestampTag::ReadStart => 2,
            TimestampTag::ReadEnd => 3,
            TimestampTag::WriteSyncStart => 4,
            TimestampTag::WriteSyncEnd => 5,
            TimestampTag::WriteStart => 6,
            TimestampTag::WriteEnd => 7,
        };
        res[1] = self.ku.into();
        res[2..2 + 8].copy_from_slice(self.ticker.to_ne_bytes().as_slice());
        res[2 + 8..2 + 8 + 8].copy_from_slice(self.time.to_ne_bytes().as_slice());
        res
    }
}

// pub(crate) struct TimestampRecorder {
//     pub(crate) records: Vec<Timestamp, VAllocator>,
// }

// impl Deref for TimestampRecorder {
//     type Target = Vec<Timestamp, VAllocator>;

//     fn deref(&self) -> &Self::Target {
//         &self.records
//     }
// }

// impl DerefMut for TimestampRecorder {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.records
//     }
// }

// impl TimestampRecorder {
//     pub(crate) fn new() -> TimestampRecorder {
//         let records = Vec::<Timestamp, VAllocator>::new_in(VAllocator);
//         TimestampRecorder { records }
//     }

//     pub(crate) fn push(&self, ts: Timestamp) {
//         let this = unsafe { &mut *(self as *const TimestampRecorder).cast_mut() };
//         this.records.try_push(ts).expect("failed to push");
//     }

//     pub(crate) fn len(&self) -> usize {
//         self.records.len()
//     }

//     pub(crate) fn into_raw(&self) -> Vec<u8, VAllocator> {
//         let mut res = Vec::<u8, VAllocator>::new_in(VAllocator);
//         for ts in &self.records {
//             res.try_extend_from_slice(ts.into_raw().as_slice()).unwrap();
//         }
//         res
//     }
// }
