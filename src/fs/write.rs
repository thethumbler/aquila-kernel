use prelude::*;
use fs::*;
use dev::kdev::*;

use crate::include::core::types::*;
use crate::include::core::string::*;
use crate::include::bits::errno::*;
use crate::include::fs::vfs::*;
use crate::dev::dev::*;
use crate::include::fs::stat::*;
use crate::fs::vnode::*;

use crate::{ISDEV, VNODE_DEV, DEV_MAJOR, DEV_MINOR};
use crate::{S_ISCHR, S_ISBLK};

pub unsafe fn vfs_write(vnode: *mut Vnode, off: off_t, size: usize, buf: *mut u8) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_write(vnode=%p, off=%d, size=%d, buf=%p)\n", vnode, off, size, buf);

    /* Invalid request */
    if vnode.is_null() {
        return -EINVAL;
    }

    /* Device node */
    if ISDEV!(vnode) {
        return kdev_write(&mut VNODE_DEV!(vnode), off, size, buf);
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
