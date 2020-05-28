use prelude::*;
use fs::*;

use crate::{ISDEV};

pub unsafe fn vfs_trunc(vnode: *mut Vnode, len: off_t) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_trunc(vnode=%p, len=%d)\n", vnode, len);

    /* invalid request */
    if vnode.is_null() {
        return -EINVAL;
    }

    /* device node */
    if ISDEV!(vnode) {
        return -EINVAL;
    }

    /* invalid request */
    //if (*vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    /* operation not supported */
    //if ((*(*vnode).fs).vops.trunc as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).trunc(len);
}

