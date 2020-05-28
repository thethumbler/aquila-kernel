use prelude::*;
use fs::*;
use bits::dirent::*;

use crate::kern::print::cstr;

use crate::{ISDEV, VNODE_DEV, DEV_MAJOR, DEV_MINOR};

/** read entries from a directory vnode
 * \ingroup vfs
 */
pub unsafe fn vfs_readdir(dir: *mut Vnode, off: off_t, dirent: *mut DirectoryEntry) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_readdir(dir=%p, off=%d, dirent=%p)\n", dir, off, dirent);

    /* Invalid request */
    if dir.is_null() || (*dir).fs.is_null() {
        return -EINVAL;
    }

    if !S_ISDIR!((*dir).mode) {
        return -ENOTDIR;
    }

    /* Operation not supported */
    //if ((*(*dir).fs).vops.readdir as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*dir).readdir(off, dirent) as isize;
}

pub unsafe fn vfs_finddir(dir: *mut Vnode, name: *const u8, dirent: *mut DirectoryEntry) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_finddir(dir=%p, name=%s, dirent=%p)\n", dir, name, dirent);
    //print!("vfs_finddir(dir={:p}, name={}, dirent={:p})\n", dir, cstr(name), dirent);

    if dir.is_null() || (*dir).fs.is_null() {
        return -EINVAL;
    }

    if !S_ISDIR!((*dir).mode) {
        return -ENOTDIR;
    }

    //if ((*(*dir).fs).vops.finddir as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*dir).finddir(name, dirent);
}
