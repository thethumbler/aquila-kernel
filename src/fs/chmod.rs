use prelude::*;
use fs::*;

use crate::{ISDEV};

pub unsafe fn vfs_chmod(vnode: *mut Vnode, mode: mode_t) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_chmod(vnode=%p, mode=%x)\n", vnode, mode);

    /* invalid request */
    if vnode.is_null() {
        return -EINVAL;
    }

    /* device node */
    if ISDEV!(vnode) {
        return -EINVAL;
    }

    /* invalid request */
    if (*vnode).fs.is_null() {
        return -EINVAL;
    }

    /* operation not supported */
    //if ((*(*vnode).fs).vops.chmod as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).chmod(mode);
}
