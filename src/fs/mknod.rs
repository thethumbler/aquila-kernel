use prelude::*;
use fs::*;
use bits::dirent::*;
use mm::*;

use crate::{ISDEV, VNODE_DEV, DEV_MAJOR, DEV_MINOR};

pub unsafe fn vfs_mknod(path: *const u8, mode: mode_t, dev: dev_t, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    let mut ret = 0;

    let mut p: *mut VfsPath = core::ptr::null_mut();
    let mut tokens: *mut *mut u8 = core::ptr::null_mut();

    /* if path is NULL pointer, or path is empty string, return NULL */
    if path.is_null() || *path == 0 {
        return -ENOENT;
    }

    let mut _path: *mut u8 = core::ptr::null_mut();
    ret = vfs_parse_path(path, uio, &mut _path);
    if ret != 0 {
        //goto error;
        // FIXME
        return ret;
    }

    /* canonicalize path */
    tokens = tokenize_path(_path);

    /* get mountpoint & path */
    p = vfs_get_mountpoint(tokens);

    let mut dir: *mut Vnode = (*p).root;
    let mut name: *mut u8 = core::ptr::null_mut();
    let mut tok: *mut *mut u8 = (*p).tokens;

    while !tok.is_null() {
        let token = *tok;

        if (*tok.offset(1)).is_null() {
            name = token;
            break;
        }

        let mut dirent: DirectoryEntry = core::mem::uninitialized();
        ret = vfs_finddir(dir, token, &mut dirent);
        if ret != 0 {
            //goto error;
            //FIXME
            return ret;
        }

        ret = vfs_vget((*p).root, dirent.d_ino, &mut dir);
        if ret != 0 {
            //goto error;
            //FIXME
            return ret;
        }

        tok = tok.offset(1);
    }

    ret = vfs_vmknod(dir, name, mode, dev, uio, vnode_ref);
    if ret != 0 {
        //goto error;
        //FIXME
        return ret;
    }

    free_tokens(tokens);
    kfree(p as *mut u8);
    kfree(_path);

    return 0;
}

pub unsafe fn vfs_mkdir(path: *const u8, mode: mode_t, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    return vfs_mknod(path, S_IFDIR | mode, 0, uio, vnode_ref);
}

pub unsafe fn vfs_creat(path: *const u8, mode: mode_t, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    return vfs_mknod(path, S_IFREG | mode, 0, uio, vnode_ref);
}
