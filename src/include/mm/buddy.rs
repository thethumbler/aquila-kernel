use prelude::*;

use crate::include::mm::vm::*;
use crate::include::core::types::*;

pub const BUDDY_MAX_ORDER: usize = 10;
pub const BUDDY_MIN_BS:    usize = 4096;
pub const BUDDY_MAX_BS:    usize = BUDDY_MIN_BS << BUDDY_MAX_ORDER;

pub const BUDDY_ZONE_NR:     usize = 2;
pub const BUDDY_ZONE_DMA:    usize = 0;
pub const BUDDY_ZONE_NORMAL: usize = 1;
