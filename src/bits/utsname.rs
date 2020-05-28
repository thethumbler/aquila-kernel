use prelude::*;

pub const MAX_LENGTH: usize = 64;

pub struct UtsName {
    pub sysname:  [u8; MAX_LENGTH],
    pub nodename: [u8; MAX_LENGTH],
    pub release:  [u8; MAX_LENGTH],
    pub version:  [u8; MAX_LENGTH],
    pub machine:  [u8; MAX_LENGTH],
}
