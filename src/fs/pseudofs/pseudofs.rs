use prelude::*;
use fs::*;
use mm::*;
use bits::dirent::*;
use kern::time::*;
use crate::{malloc_define, malloc_declare};

malloc_define!(M_PSEUDOFS_DENT, "pseudofs-dirent\0", "pseudofs directory entry\0");
malloc_declare!(M_VNODE);

pub struct PseudofsDirent {
    pub d_name: *const u8,
    pub d_ino: *mut Vnode,

    pub next: *mut PseudofsDirent,
}

#[no_mangle]
pub unsafe fn pseudofs_vmknod(dir: *mut Vnode, name: *const u8, mode: mode_t, dev: dev_t, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    let mut err = 0;

    let mut vnode = core::ptr::null_mut() as *mut Vnode;
    let mut dirent = core::ptr::null_mut() as *mut PseudofsDirent;

    let mut dirent_tmp: DirectoryEntry = core::mem::uninitialized();
    if pseudofs_finddir(dir, name, &dirent_tmp as *const _ as *mut DirectoryEntry) == 0 {
        err = -EEXIST;
        return err;
    }

    vnode = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, M_ZERO) as *mut Vnode;
    if vnode.is_null() {
        err = -ENOMEM;
        return err;
    }
    
    (*vnode).ino   = vnode as ino_t;
    (*vnode).mode  = mode;
    (*vnode).size  = 0;
    (*vnode).uid   = (*uio).uid;
    (*vnode).gid   = (*uio).gid;
    (*vnode).nlink = if S_ISDIR!(mode) { 2 } else { 1 };
    (*vnode).rdev  = dev;

    let mut ts: TimeSpec = core::mem::uninitialized();
    gettime(&ts as *const _ as *mut TimeSpec);

    (*vnode).ctime = ts;
    (*vnode).atime = ts;
    (*vnode).mtime = ts;

    /* copy filesystem from directory */
    (*vnode).fs = (*dir).fs;

    let mut cur_dir = (*dir).p as *mut PseudofsDirent;

    dirent = kmalloc(core::mem::size_of::<PseudofsDirent>(), &M_PSEUDOFS_DENT, 0) as *mut PseudofsDirent;
    if dirent.is_null() {
        // TODO
        err = -ENOMEM;
        return err;
    }

    (*dirent).d_ino  = vnode;
    (*dirent).d_name = strdup(name);

    if (*dirent).d_name.is_null() {
        // TODO
        err = -ENOMEM;
        return err;
    }

    (*dirent).next  = cur_dir;
    (*dir).p        = dirent as *mut u8;

    if !vnode_ref.is_null() {
        *vnode_ref = vnode;
    }

    return 0;

//error_nomem:
//    err = -ENOMEM;
//    goto error;
//
//error:
//    if (vnode)
//        kfree(vnode);
//
//    if (dirent) {
//        if (dirent->d_name)
//            kfree((void *) dirent->d_name);
//
//        kfree(dirent);
//    }
//
//    return err;
}

#[no_mangle]
pub unsafe fn pseudofs_vunlink(vnode: *mut Vnode, name: *const u8, uio: *mut UserOp) -> isize {
    let mut err = 0;

    let mut dir = core::ptr::null_mut() as *mut PseudofsDirent;
    let mut next = core::ptr::null_mut() as *mut Vnode;
    let mut prev = core::ptr::null_mut();
    let mut cur = core::ptr::null_mut();

    if !S_ISDIR!((*vnode).mode) {
        return -ENOTDIR;
    }

    dir   = (*vnode).p as *mut PseudofsDirent;

    if dir.is_null() {
        /* directory not initialized */
        return -ENOENT;
    }

    prev = core::ptr::null_mut();
    cur  = core::ptr::null_mut();

    let mut found = false;
    let mut dirent = dir; 
    while !dirent.is_null() {
        if strcmp((*dirent).d_name, name) == 0 {
            cur = dirent;
            found = true;
            break;
        }

        prev = dirent;
        dirent = (*dirent).next; 
    }

    if !found {
        /* file not found */
        return -ENOENT;
    }

    if !prev.is_null() {
        (*prev).next = (*cur).next;
    } else {
        (*vnode).p   = (*cur).next as *mut u8;
    }

    (*(*cur).d_ino).nlink = 0;

    if (*(*cur).d_ino).refcnt == 0 {
        /* vfs_close will decrement ref */
        (*(*cur).d_ino).refcnt = 1;
        vfs_close((*cur).d_ino);
    }

    return 0;
}

#[no_mangle]
pub unsafe fn pseudofs_readdir(dir: *mut Vnode, offset: off_t, dirent: *mut DirectoryEntry) -> usize {
    let mut offset = offset;

    if offset == 0 {
        strcpy(&(*dirent).d_name as *const _ as *mut u8, b".\0".as_ptr());
        return 1;
    }

    if offset == 1 {
        strcpy(&(*dirent).d_name as *const _ as *mut u8, b"..\0".as_ptr());
        return 1;
    }

    let _dirent = (*dir).p as *mut PseudofsDirent;

    if _dirent.is_null() {
        return 0;
    }

    let mut found = 0;

    offset -= 2;

    let mut i = 0;
    let mut e = _dirent; 
    while !e.is_null() {
        if i == offset {
            found = 1;
            (*dirent).d_ino = (*(*e).d_ino).ino;
            strcpy(&(*dirent).d_name as *const _ as *mut u8, (*e).d_name);
            break;
        }

        i += 1;
        e = (*e).next;
    }

    return found;
}

#[no_mangle]
pub unsafe fn pseudofs_finddir(dir: *mut Vnode, name: *const u8, dirent: *mut DirectoryEntry) -> isize {
    if dir.is_null() || name.is_null() || dirent.is_null() {
        return -EINVAL;
    }

    if !S_ISDIR!((*dir).mode) {
        return -ENOTDIR;
    }

    let _dirent = (*dir).p as *mut PseudofsDirent;
    if _dirent.is_null() {
        return -ENOENT;
    }

    let mut e = _dirent;
    while !e.is_null() {
        if strcmp(name, (*e).d_name) == 0 {
            (*dirent).d_ino = (*(*e).d_ino).ino;
            strcpy(&(*dirent).d_name as *const _ as *mut u8, (*e).d_name);
            return 0;
        }

        e = (*e).next;
    }

    return -ENOENT;
}

#[no_mangle]
pub unsafe fn pseudofs_close(vnode: *mut Vnode) -> isize {
    /* XXX */
    //kfree(inode->name);
    kfree(vnode as *mut u8);
    return 0;
}
