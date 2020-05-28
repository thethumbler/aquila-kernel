use prelude::*;

use kern::time::*;
use crate::include::core::types::*;
use crate::include::fs::vfs::*;
use crate::fs::vfs::*;
use crate::include::mm::kvmem::*;
use crate::include::bits::errno::*;
use crate::fs::tmpfs::tmpfs::*;
use crate::fs::vnode::*;
use crate::include::fs::stat::*;
use crate::include::core::time::*;
use crate::include::core::module::*;

use crate::{malloc_declare};

malloc_declare!(M_VNODE);

/* devfs root directory (usually mounted on '/dev') */
pub static mut devfs_root: *mut Vnode = core::ptr::null_mut();

unsafe fn devfs_init() -> isize {
    /* devfs is really just tmpfs */
    devfs.vops = tmpfs.vops.clone();
    devfs.fops = tmpfs.fops.clone();

    devfs_root = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, M_ZERO) as *mut Vnode;
    if devfs_root.is_null() {
        return -ENOMEM;
    }

    (*devfs_root).ino    = devfs_root as usize as ino_t;
    (*devfs_root).mode   = S_IFDIR | 0775;
    (*devfs_root).nlink  = 2;
    (*devfs_root).fs     = &devfs;
    (*devfs_root).refcnt = 1;

    let mut ts: TimeSpec = core::mem::uninitialized();
    gettime(&mut ts);

    (*devfs_root).ctime = ts;
    (*devfs_root).atime = ts;
    (*devfs_root).mtime = ts;

    vfs_install(&mut devfs);

    return 0;
}

unsafe fn devfs_mount(dir: *const u8, flags: isize, data: *mut u8) -> isize {
    if devfs_root.is_null() {
        return -EINVAL;
    }

    vfs_bind(dir, devfs_root)
}

pub static mut devfs: Filesystem = Filesystem {
    name:  "devfs",
    nodev: 1,

    _init:  Some(devfs_init),
    _mount: Some(devfs_mount),
    _load: None,

    vops: VnodeOps::empty(),
    fops: FileOps::none(),
};

module_init!(devfs, Some(devfs_init), None);
