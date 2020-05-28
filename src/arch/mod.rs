#[cfg(target_arch = "x86")]
pub mod i386;

#[cfg(target_arch = "x86")]
pub use self::i386::*;

pub struct Arch {}
