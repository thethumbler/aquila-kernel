use prelude::*;
use fs::*;

pub unsafe fn rofs_write(vnode: *mut Vnode, offset: off_t, size: usize, buf: *mut u8) -> usize {
    return -EROFS as usize;
}

pub unsafe fn rofs_trunc(vnode: *mut Vnode, len: off_t) -> isize {
    return -EROFS;
}

pub unsafe fn rofs_vmknod(dir: *mut Vnode, name: *const u8, mode: mode_t, dev: dev_t, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    return -EROFS;
}

pub unsafe fn rofs_vunlink(dir: *mut Vnode, name: *const u8, uio: *mut UserOp) -> isize {
    return -EROFS;
}
