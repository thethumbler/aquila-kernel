use prelude::*;

use crate::include::mm::vm::*;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_MASK: usize = 4096 - 1;

#[macro_export]
macro_rules! page_align {
    ($ptr:expr) => {
        (($ptr as usize) & !PAGE_MASK)
    }
}

#[macro_export]
macro_rules! page_round {
    ($ptr:expr) => {
        ((($ptr as usize) + PAGE_MASK) & !PAGE_MASK)
    }
}

//#include <boot/boot.h>

pub const PF_PRESENT: usize = 0x001;
pub const PF_READ:    usize = 0x002;
pub const PF_WRITE:   usize = 0x004;
pub const PF_EXEC:    usize = 0x008;
pub const PF_USER:    usize = 0x010;
