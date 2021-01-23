use prelude::*;
use arch;

/* XXX use a better name */
pub fn gettime() -> Result<TimeSpec, Error> {
    // XXX wrap this in arch::time
    arch::misc::cmos::gettime()
}

pub fn gettimeofday() -> Result<(TimeVal, TimeZone), Error> {
    let ts = gettime()?;

    let tv = TimeVal {
        tv_sec: ts.tv_sec,
        tv_usec: (ts.tv_nsec / 1000) as usize,
    };

    let tz = TimeZone {
        tz_minuteswest: 0,
        tz_dsttime: 0,
    };

    Ok((tv, tz))
}

