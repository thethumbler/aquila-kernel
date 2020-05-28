use prelude::*;

use crate::include::core::types::*;
use crate::include::core::string::*;
use crate::include::bits::errno::*;
use crate::include::fs::vfs::*;
use crate::include::fs::stat::*;
use crate::fs::vnode::*;

/* closes a vnode (i.e. decrements reference counts) */
pub unsafe fn vfs_close(vnode: *mut Vnode) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_close(vnode=%p)\n", vnode);

    /* invalid request */
    if vnode.is_null() || (*vnode).fs.is_null() {
        return -EINVAL;
    }

    /* operation not supported */
    //if ((*(*vnode).fs).vops.close as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    if (*vnode).refcnt == 0 {
        panic!("closing an already closed vnode");
    }

    (*vnode).refcnt -= 1;

    if (*vnode).refcnt == 0 {
        return (*vnode).close();
    }

    return 0;
}
