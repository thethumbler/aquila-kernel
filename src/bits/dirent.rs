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

impl DirectoryEntry {
    pub const fn none() -> Self {
        DirectoryEntry {
            d_ino: 0,
            d_name: [0; MAXNAMELEN],
        }
    }

    pub fn new(d_ino: ino_t, name: &str) -> Self {
        let mut d_name = [0u8; MAXNAMELEN];
        unsafe { memcpy(d_name.as_mut_ptr(), name.as_ptr(), name.len()) };

        DirectoryEntry { d_ino, d_name }
    }
}
