use prelude::*;

use crate::arch::i386::platform::misc::cmos::arch_time_get;

/* XXX use a better name */
pub unsafe fn gettime(ts: *mut TimeSpec) -> isize {
    return arch_time_get(ts);
}

pub unsafe fn gettimeofday(tv: *mut TimeVal, tz: *mut TimeZone) -> isize {
    let mut err = 0;

    let mut ts: TimeSpec = core::mem::uninitialized();
    err = gettime(&mut ts);

    if err != 0 {
        return err;
    }

    if !tz.is_null() {
        (*tz).tz_minuteswest = 0;
        (*tz).tz_dsttime = 0;
    }

    if !tv.is_null() {
        (*tv).tv_sec  = ts.tv_sec;
        (*tv).tv_usec = (ts.tv_nsec / 1000) as usize;
    }

    return 0;
}

