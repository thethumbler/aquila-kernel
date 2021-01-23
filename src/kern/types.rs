use prelude::*;

pub type ssize_t = isize;
pub type size_t = usize;

pub type pid_t = isize;
pub type tid_t = pid_t;
pub type off_t = isize;

pub type vino_t  = usize;
pub type mode_t  = u32;
pub type mask_t  = u8;
pub type devid_t = u8;
pub type dev_t   = u16;

pub type uid_t   = u32;
pub type gid_t   = u32;
pub type nlink_t = u32;
pub type ino_t   = usize;

pub type _time_t = TimeSpec;

pub type time_t = u64;
pub type sigset_t = usize;
pub type suseconds_t = usize;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TimeSpec {
    pub tv_sec: time_t,
    pub tv_nsec: u32,
}

impl TimeSpec {
    pub const fn none() -> Self {
        TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        }
    }
}

#[repr(C)]
pub struct TimeVal {
    pub tv_sec: time_t,         /* seconds */
    pub tv_usec: suseconds_t,   /* microseconds */
}

#[repr(C)]
pub struct TimeZone {
    pub tz_minuteswest: isize,     /* minutes west of Greenwich */
    pub tz_dsttime: isize,         /* type of DST correction */
}

#[repr(C)]
pub struct utimbuf {
    pub actime: time_t,
    pub modtime: time_t,
}
