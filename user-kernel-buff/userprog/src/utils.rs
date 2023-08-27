use std::mem::size_of;

use libc::{c_int, c_ulong, ioctl};

use crate::timestamp::Timestamp;

// https://elixir.bootlin.com/linux/v5.12.9/source/include/uapi/asm-generic/ioctl.h#L88
const IOC_NRBITS: u32 = 8;
const IOC_TYPEBITS: u32 = 8;
const IOC_SIZEBITS: u32 = 14;
const IOC_DIRBITS: u32 = 2;
const IOC_NRSHIFT: u32 = 0;
const IOC_TYPESHIFT: u32 = IOC_NRSHIFT + IOC_NRBITS;
const IOC_SIZESHIFT: u32 = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT: u32 = IOC_SIZESHIFT + IOC_SIZEBITS;

pub(crate) fn ioc(dir: u32, typ: u32, nr: u32, size: u32) -> c_ulong {
    let res = (dir << IOC_DIRSHIFT)
        | (typ << IOC_TYPESHIFT)
        | (nr << IOC_NRSHIFT)
        | (size << IOC_SIZESHIFT);
    res as _
}

pub(crate) fn iorw(typ: u32, nr: u32, size: u32) -> c_ulong {
    ioc(3, typ, nr, size)
}

pub(crate) fn ior(typ: u32, nr: u32, size: u32) -> c_ulong {
    ioc(2, typ, nr, size)
}

pub(crate) fn ion(typ: u32, nr: u32, size: u32) -> c_ulong {
    ioc(0, typ, nr, size)
}

pub(crate) fn get_kern_timestamps(fd: c_int) -> Vec<Timestamp> {
    // get the number of timestamps
    let ts_cnt_raw = [0u8; 4];
    unsafe {
        ioctl(
            fd,
            ior(107, 0, size_of::<*mut u32>() as u32) as _,
            ts_cnt_raw.as_ptr(),
        );
    }
    let ts_cnt = u32::from_ne_bytes(ts_cnt_raw);

    // buffer for raw timestamps
    // each raw timestamps is 18 bytes:
    //   1(tag) + 1(kern/user) + 8(ticker) + 8(time_in_ns)
    let mut raw_vec: Vec<u8> = Vec::new();
    raw_vec.resize(18 * ts_cnt as usize, 0);

    // get raw timestamps from kernel space
    unsafe {
        ioctl(fd, ion(107, 1, 0) as _, raw_vec.as_mut_ptr());
    }

    let ts_vec: Vec<Timestamp> = raw_vec
        .chunks(18)
        .map(|chunk| Timestamp::from_bytes(chunk))
        .collect();
    ts_vec
}
