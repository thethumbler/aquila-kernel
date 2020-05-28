use prelude::*;

use kern::string::*;
use arch::i386::earlycon::earlycon::earlycon_putc;

pub struct Console {}

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.as_bytes() {
            unsafe {
                earlycon_putc(*c);
            }
        }

        Ok(())
    }
}

pub static mut rconsole: Console = Console {};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        unsafe {
            core::fmt::write(&mut $crate::kern::print::rconsole, core::format_args!($($arg)*))
        }
    };
}

pub fn cstr(p: *const u8) -> &'static str {
    unsafe {
        let len = strlen(p);
        let s = core::slice::from_raw_parts(p, len);

        core::str::from_utf8(s).unwrap()
    }
}
