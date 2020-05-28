use prelude::*;
use fs::*;
use crate::{ISDEV};

pub unsafe fn vfs_chown(vnode: *mut Vnode, uid: uid_t, gid: gid_t) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_chown(vnode=%p, uid=%d, gid=%d)\n", vnode, uid, gid);

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
    //if ((*(*vnode).fs).vops.chown as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).chown(uid, gid);
}
