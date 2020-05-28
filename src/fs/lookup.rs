use prelude::*;

use fs::*;

use kern::string::*;
use crate::include::core::types::*;
use crate::include::bits::errno::*;
use crate::include::bits::dirent::*;
use crate::include::bits::fcntl::*;
use crate::include::fs::vfs::*;
use crate::include::fs::stat::*;
use crate::include::mm::kvmem::*;
use crate::fs::vnode::*;
use crate::fs::read::*;
use crate::mm::kvmem::M_BUFFER;
use crate::kern::print::cstr;

use crate::{ISDEV, VNODE_DEV, DEV_MAJOR, DEV_MINOR};
use crate::{S_ISCHR, S_ISBLK, S_ISLNK, print};

unsafe fn vfs_follow(vnode: *mut Vnode, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    /* TODO enforce limit */

    let mut err = 0;
    let mut path: *mut u8 = core::ptr::null_mut();
    
    path = kmalloc(1024, &M_BUFFER, M_ZERO);

    if path.is_null() {
        return -ENOMEM;
    }

    err = vfs_read(vnode, 0, 1024, path);
    if err < 0 {
        kfree(path as *mut u8);
        return err;
    }

    err = vfs_lookup(path, uio, vnode_ref, core::ptr::null_mut());
    if err != 0 {
        kfree(path as *mut u8);
        return err;
    }

    kfree(path as *mut u8);

    return 0;
}

pub unsafe fn vfs_lookup(path: *const u8, uio: *mut UserOp, vnode_ref: *mut *mut Vnode, abs_path: *mut *mut u8) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_lookup(path=%s, uio=%p, ref=%p, abs_path=%p)\n", path, uio, vnode_ref, abs_path);
    
    //print!("vfs_lookup(path={}, uio={:p}, ref={:p}, abs_path={:p})\n", cstr(path), uio, vnode_ref, abs_path);

    let mut ret = 0;

    let mut vfs_path: *mut VfsPath = core::ptr::null_mut();
    let mut tokens = core::ptr::null_mut();

    if path.is_null() || *path == 0 {
        return -ENOENT;
    }

    /* get real path (i.e. without . or ..) */
    let mut rpath = core::ptr::null_mut();
    ret = vfs_parse_path(path, uio, &mut rpath);

    if ret != 0 {
        return ret;
    }

    tokens = tokenize_path(rpath);

    /* Get mountpoint & path */
    vfs_path = vfs_get_mountpoint(tokens);

    let mut dir = (*vfs_path).root;
    (*dir).refcnt += 1;

    let mut token_p = (*vfs_path).tokens;
    while !(*token_p).is_null() {
        let token = *token_p;

        let mut dirent: DirectoryEntry = core::mem::uninitialized();
        ret = vfs_finddir(dir, token, &mut dirent);
        if ret != 0 {
            break;
        }

        ret = vfs_vget((*vfs_path).root, dirent.d_ino, &mut dir);
        if ret != 0 {
            break;
        }

        token_p = token_p.offset(1);
    }

    if ret != 0 {
        if !tokens.is_null() {
            free_tokens(tokens);
        }

        if !vfs_path.is_null() {
            kfree(vfs_path as *mut u8);
        }

        if !rpath.is_null() {
            kfree(rpath as *mut u8);
        }

        return ret;
    }

    free_tokens(tokens);
    kfree(vfs_path as *mut u8);

    if !vnode_ref.is_null() {
        *vnode_ref = dir;
    }

    if !abs_path.is_null() {
        *abs_path = strdup(rpath);
    }

    kfree(rpath as *mut u8);

    /* resolve symbolic links */
    if S_ISLNK!((*dir).mode) && ((*uio).flags as usize & O_NOFOLLOW == 0) {
        return vfs_follow(dir, uio, vnode_ref);
    }

    return 0;
}
