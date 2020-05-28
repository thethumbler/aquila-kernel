use prelude::*;
use fs::*;
use fs::tmpfs::*;
use mm::*;
use kern::time::*;

use crate::{malloc_declare};

malloc_declare!(M_VNODE);

/* devfs root directory (usually mounted on '/dev') */
pub static mut DEVFS_ROOT: *mut Vnode = core::ptr::null_mut();

unsafe fn devfs_init() -> isize {
    /* devfs is really just tmpfs */
    DEVFS.vops = TMPFS.vops.clone();
    DEVFS.fops = TMPFS.fops.clone();

    DEVFS_ROOT = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, M_ZERO) as *mut Vnode;
    if DEVFS_ROOT.is_null() {
        return -ENOMEM;
    }

    (*DEVFS_ROOT).ino    = DEVFS_ROOT as usize as ino_t;
    (*DEVFS_ROOT).mode   = S_IFDIR | 0775;
    (*DEVFS_ROOT).nlink  = 2;
    (*DEVFS_ROOT).fs     = &DEVFS;
    (*DEVFS_ROOT).refcnt = 1;

    let mut ts: TimeSpec = core::mem::uninitialized();
    gettime(&mut ts);

    (*DEVFS_ROOT).ctime = ts;
    (*DEVFS_ROOT).atime = ts;
    (*DEVFS_ROOT).mtime = ts;

    vfs_install(&mut DEVFS);

    return 0;
}

unsafe fn devfs_mount(dir: *const u8, flags: isize, data: *mut u8) -> isize {
    if DEVFS_ROOT.is_null() {
        return -EINVAL;
    }

    vfs_bind(dir, DEVFS_ROOT)
}

pub static mut DEVFS: Filesystem = Filesystem {
    name:  "devfs",
    nodev: 1,

    _init:  Some(devfs_init),
    _mount: Some(devfs_mount),
    _load: None,

    vops: VnodeOps::empty(),
    fops: FileOps::none(),
};

module_init!(devfs, Some(devfs_init), None);
