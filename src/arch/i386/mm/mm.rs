use prelude::*;

use crate::arch::i386::mm::i386::pmap_init;

pub unsafe fn arch_mm_setup() {
    pmap_init();
}

