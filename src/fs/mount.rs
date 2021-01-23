use prelude::*;

use fs::{self, *};
use mm::*;

use kern::print::cstr;
use crate::{print, malloc_define};

malloc_define!(M_MOUNTPOINT, "mountpoint\0", "mount point structure\0");

pub fn mount(fs_type: &str, dir: &str, flags: isize, data: *mut u8, uio: &UserOp) -> Result<(), Error> {
    unsafe {
        match REGISTERED_FS.iter().find(|fs| fs.name == fs_type) {
            None => Err(Error::EINVAL),
            Some(fs) => {
                let realpath = fs::realpath(dir, uio)?;
                fs.mount(Arc::clone(fs), &realpath, flags, data)
            }
        }
    }
}

pub fn get_fs_by_name(name: &str) -> Option<Arc<Filesystem>> {
    unsafe {
        REGISTERED_FS.iter().find(|fs| fs.name == name)
            .map(|fs| Arc::clone(fs))
    }
}