use prelude::*;

pub const PROT_NONE   : usize = 0x00000;
pub const PROT_READ   : usize = 0x00001;
pub const PROT_WRITE  : usize = 0x00002;
pub const PROT_EXEC   : usize = 0x00004;

pub const MAP_FAILED  : usize = 0;
pub const MAP_FIXED   : usize = 0x00001;
pub const MAP_PRIVATE : usize = 0x00002;
pub const MAP_SHARED  : usize = 0x00004;
