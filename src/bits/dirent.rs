use prelude::*;

pub const MAXNAMELEN: usize = 256;

/**
 * \ingroup vfs
 * \brief directory entry
 */
#[repr(C)]
pub struct DirectoryEntry {
    pub d_ino: usize,
    pub d_name: [u8; MAXNAMELEN],
}
