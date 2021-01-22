use prelude::*;
use fs::vnode::*;

/* sync the metadata and/or data associated with a vnode */
pub unsafe fn vfs_vsync(vnode: *mut Vnode, mode: isize) -> isize {
    return -Error::ENOTSUP;
}

/* sync the metadata and/or data associated with a filesystem */
pub unsafe fn vfs_fssync(super_node: *mut Vnode, mode: isize) -> isize {
    return -Error::ENOTSUP;
}

/* sync all metadata and/or data of all filesystems */
pub unsafe fn vfs_sync(mode: isize) -> isize {
    return -Error::ENOTSUP;
}
