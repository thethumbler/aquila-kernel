use prelude::*;

use crate::include::core::types::TimeSpec;
use crate::arch::i386::include::cpu::io::IOAddr;
use crate::include::core::types::time_t;
use crate::{print};

const RTC_SEC: u8 = 0x00;     /* seconds */
const RTC_MIN: u8 = 0x02;     /* minutes */
const RTC_HRS: u8 = 0x04;     /* hours */
const RTC_WD:  u8 = 0x06;     /* weekday */
const RTC_DOM: u8 = 0x07;     /* day of month */
const RTC_MON: u8 = 0x08;     /* month */
const RTC_YR:  u8 = 0x09;     /* year */
const RTC_SA:  u8 = 0x0A;     /* status register a */
const RTC_SB:  u8 = 0x0B;     /* status register b */
const RTC_BIN: u8 = 0x04;     /* binary mode */

static mut cmos: IOAddr = IOAddr::empty();

unsafe fn cmos_reg_read(reg: u8) -> u8 {
    cmos.out8(0, (1 << 7) | (reg));
    return cmos.in8(1);
}

#[repr(C)]
struct CmosRtc {
    yr: u8,
    mon: u8,
    mday: u8,
    hrs: u8,
    min: u8,
    sec: u8,
    wday: u8,
}

#[inline]
unsafe fn bcd_to_bin(bcd: u8) -> u8 {
    return ((bcd >> 4) & 0xF) * 10 + (bcd & 0xF);
}

unsafe fn cmos_time(rtc: *mut CmosRtc) -> isize {
    let fmt = cmos_reg_read(RTC_SB);

    (*rtc).yr   = cmos_reg_read(RTC_YR);
    (*rtc).mon  = cmos_reg_read(RTC_MON);
    (*rtc).mday = cmos_reg_read(RTC_DOM);
    (*rtc).hrs  = cmos_reg_read(RTC_HRS);
    (*rtc).min  = cmos_reg_read(RTC_MIN);
    (*rtc).sec  = cmos_reg_read(RTC_SEC);
    (*rtc).wday = cmos_reg_read(RTC_WD);

    if fmt & RTC_BIN == 0 {
        /* convert all values to binary */
        (*rtc).yr   = bcd_to_bin((*rtc).yr);
        (*rtc).mon  = bcd_to_bin((*rtc).mon);
        (*rtc).mday = bcd_to_bin((*rtc).mday);
        (*rtc).hrs  = bcd_to_bin((*rtc).hrs);
        (*rtc).min  = bcd_to_bin((*rtc).min);
        (*rtc).sec  = bcd_to_bin((*rtc).sec);
        (*rtc).wday = bcd_to_bin((*rtc).wday);
    }

    return 0;
}

/* FIXME: should be elsewhere */
pub unsafe fn arch_time_get(ts: *mut TimeSpec) -> isize {
    let mut rtc: CmosRtc = core::mem::uninitialized();
    cmos_time(&mut rtc);

    let mut time: time_t = 0;

    let yr = rtc.yr as u64 + 2000;
    let mon = rtc.mon as u64;
    let mday = rtc.mday as u64;
    let hrs = rtc.hrs as u64;
    let min = rtc.min as u64;
    let sec = rtc.sec as u64;

    /* convert years to days */
    time = (365 * yr) + (yr / 4) - (yr / 100) + (yr / 400);
    /* convert months to days */
    time += (30 * mon) + (3 * (mon + 1) / 5) + mday;
    /* UNIX time starts on January 1st, 1970 */
    time -= 719561;
    /* convert days to seconds */
    time *= 86400;
    /* add hours, minutes and seconds */
    time += (3600 * hrs) + (60 * min) + sec;

    (*ts).tv_sec = time;
    (*ts).tv_nsec = 0;

    return 0;
}

pub unsafe fn x86_cmos_setup(ioaddr: *mut IOAddr) -> isize {
    cmos = *ioaddr;
    print!("cmos: initializing [{:p} ({})]\n", cmos.addr as *const u8, cmos.type_str());

    return 0;
}
