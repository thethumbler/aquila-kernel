use prelude::*;

use fs::*;
use dev::*;
use dev::kdev::*;

pub unsafe fn vfs_write(vnode: *mut Vnode, off: off_t, size: usize, buf: *mut u8) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_write(vnode=%p, off=%d, size=%d, buf=%p)\n", vnode, off, size, buf);

    /* Invalid request */
    if vnode.is_null() {
        return -EINVAL;
    }

    /* Device node */
    if (*vnode).is_device() {
        return kdev_write(&mut vnode_dev!(vnode), off, size, buf);
    }

    /* Invalid request */
    //if (*vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    /* Operation not supported */
    //if ((*(*vnode).fs).vops.write as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).write(off, size, buf) as isize;
}
