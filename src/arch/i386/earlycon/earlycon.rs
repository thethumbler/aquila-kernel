use prelude::*;

use kern::kargs::kargs_get;
use arch::i386::earlycon::uart::*;

#[repr(C)]
pub struct EarlyConsole {
    pub _init: Option<unsafe fn()>,
    pub _puts: Option<unsafe fn(*const u8) -> isize>,
    pub _putc: Option<unsafe fn(u8) -> isize>,
}

impl EarlyConsole {
    pub unsafe fn init(&self) {
        self._init.unwrap()()
    }

    pub unsafe fn puts(&self, s: *const u8) -> isize {
        self._puts.unwrap()(s)
    }

    pub unsafe fn putc(&self, c: u8) -> isize {
        self._putc.unwrap()(c)
    }
}

static mut EARLYCON: *mut EarlyConsole = core::ptr::null_mut();

pub unsafe fn earlycon_puts(s: *const u8) -> isize {
    return (*EARLYCON).puts(s);
}

pub unsafe fn earlycon_putc(c: u8) -> isize {
    return (*EARLYCON).putc(c);
}

pub unsafe fn earlycon_init() {
    EARLYCON = &mut EARLYCON_UART;
    (*EARLYCON).init();
}

pub unsafe fn earlycon_reinit() {
    let mut arg_earlycon: *const u8 = core::ptr::null();

    if kargs_get("earlycon\0".as_bytes().as_ptr(), &mut arg_earlycon) == 0 {
        if strcmp(arg_earlycon, "ttyS0\0".as_bytes().as_ptr()) == 0 {
            EARLYCON = &mut EARLYCON_UART;
        }
        /*else if strcmp(arg_earlycon, "vga\0".as_bytes().as_ptr()) == 0 {
            earlycon = &mut earlycon_vga;
        } else if strcmp(arg_earlycon, "fb\0".as_bytes().as_ptr()) == 0 {
            earlycon = &mut earlycon_fb;
        } else {
            earlycon = &mut earlycon_fb; /* default */
        }
        */
    }

    (*EARLYCON).init();
}

