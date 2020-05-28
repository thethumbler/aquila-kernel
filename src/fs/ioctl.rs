use prelude::*;

use crate::include::core::types::*;
use crate::include::core::string::*;
use crate::include::bits::errno::*;
use crate::include::fs::vfs::*;
use dev::dev::*;
use dev::kdev::*;
use crate::include::fs::stat::*;
use crate::fs::vnode::*;

use crate::{ISDEV, VNODE_DEV, DEV_MAJOR, DEV_MINOR};
use crate::{S_ISCHR, S_ISBLK};

pub unsafe fn vfs_ioctl(vnode: *mut Vnode, request: usize, argp: *mut u8) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_ioctl(vnode=%p, request=%ld, argp=%p)\n", vnode, request, argp);

    /* TODO basic ioctl handling */

    /* invalid request */
    if vnode.is_null() {
        return -EINVAL;
    }

    /* device node */
    if ISDEV!(vnode) {
        return kdev_ioctl(&mut VNODE_DEV!(vnode), request as isize, argp);
    }

    /* invalid request */
    if (*vnode).fs.is_null() {
        return -EINVAL;
    }

    /* operation not supported */
    //if ((*(*vnode).fs).vops.ioctl as *mut u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).ioctl(request as isize, argp);
}
