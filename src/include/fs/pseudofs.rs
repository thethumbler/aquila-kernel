use prelude::*;

use crate::fs::vnode::*;

#[repr(C)]
pub struct PseudofsDirent {
    pub d_name: *const u8,
    pub d_ino: *mut Vnode,

    pub next: *mut PseudofsDirent,
}

//int pseudofs_vmknod(struct vnode *dir, const char *fn, mode_t mode, dev_t dev, struct uio *uio, struct vnode **ref);
//int pseudofs_vunlink(struct vnode *dir, const char *fn, struct uio *uio);
//ssize_t pseudofs_readdir(struct vnode *dir, off_t offset, struct dirent *dirent);
//int pseudofs_finddir(struct vnode *dir, const char *name, struct dirent *dirent);
//int pseudofs_close(struct vnode *vnode);
