use prelude::*;
use fs::*;

use dev::kdev::*;
use dev::*;

/* read data from a vnode */
pub unsafe fn vfs_read(vnode: *mut Vnode, off: off_t, size: usize, buf: *mut u8) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_read(vnode=%p, off=%d, size=%d, buf=%p)\n", vnode, off, size, buf);

    /* invalid request */
    if vnode.is_null() {
        return -EINVAL;
    }

    /* device node */
    if (*vnode).is_device() {
        return kdev_read(&mut vnode_dev!(vnode), off, size, buf);
    }

    /* invalid request */
    if (*vnode).fs.is_null() {
        return -EINVAL;
    }

    /* operation not supported */
    //if ((*(*vnode).fs).vops.read as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).read(off, size, buf) as isize;
}
