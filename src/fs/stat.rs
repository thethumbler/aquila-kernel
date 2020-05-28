use prelude::*;

use crate::include::core::types::*;
use crate::include::core::string::*;
use crate::include::bits::errno::*;
use crate::include::fs::vfs::*;
use crate::include::fs::stat::*;
use crate::fs::vnode::*;

use crate::{ISDEV, VNODE_DEV, DEV_MAJOR, DEV_MINOR};
use crate::{S_ISCHR, S_ISBLK};

pub unsafe fn vfs_stat(vnode: *mut Vnode, buf: *mut Stat) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_stat(vnode=%p, buf=%p)\n", vnode, buf);

    (*buf).st_dev   = (*vnode).dev;
    (*buf).st_ino   = (*vnode).ino as u16;
    (*buf).st_mode  = (*vnode).mode;
    (*buf).st_nlink = (*vnode).nlink as u16;
    (*buf).st_uid   = (*vnode).uid;
    (*buf).st_gid   = (*vnode).gid;
    (*buf).st_rdev  = (*vnode).rdev;
    (*buf).st_size  = (*vnode).size as u32;
    (*buf).st_mtime = (*vnode).mtime;
    (*buf).st_atime = (*vnode).atime;
    (*buf).st_ctime = (*vnode).ctime;

    return 0;
}
