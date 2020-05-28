use prelude::*;

use crate::kern::kargs::kargs_get;
use crate::arch::i386::include::cpu::io::IOAddr;
use crate::{print};

static mut pit_ioaddr: IOAddr = IOAddr::empty();

const PIT_CHANNEL0: usize = 0x00;
const PIT_CMD:      usize = 0x03;

pub unsafe fn x86_pit_setup(io: *mut IOAddr) -> isize {
    print!("i8254: Initializing [{:p} ({})]\n", (*io).addr as *const u8, (*io).type_str());
    pit_ioaddr = *io;
    return 0;
}

/* PIT Oscillator operates at 1.193182 MHz */
const FBASE: u32 = 1193182;

unsafe fn atou32(mut s: *const u8) -> u32 {
    let mut ret: u32 = 0;

    while *s != b'\0' {
        ret = ret * 10 + (*s - b'0') as u32;
        s = s.offset(1);
    }

    return ret;
}

pub unsafe fn x86_pit_period_set(mut period_ns: u32) -> u32 {
    print!("i8254: requested period {}ns\n", period_ns);

    let mut div;
    let mut arg_div: *const u8 = core::ptr::null_mut();

    if kargs_get(b"i8254.div\0".as_ptr(), &mut arg_div) == 0 {
        div = atou32(arg_div);
    } else {
        div = period_ns/838;
    }

    if (div == 0) {
        div = 1;
    }

    period_ns = 1000000000/(FBASE/div);

    print!("i8254: Setting period to {}ns (div = {})\n", period_ns, div);
    let cmd = 0x36;

    pit_ioaddr.out8(PIT_CMD, cmd);
    pit_ioaddr.out8(PIT_CHANNEL0, ((div >> 0) & 0xFF) as u8);
    pit_ioaddr.out8(PIT_CHANNEL0, ((div >> 8) & 0xFF) as u8);

    return period_ns;
}

