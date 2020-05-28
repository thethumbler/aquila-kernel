use prelude::*;

use kern::string::*;
use dev::rd::ramdisk::rd_size;
use crate::fs::vnode::Vnode;
use crate::include::fs::vfs::Filesystem;
use crate::fs::devfs::devfs::devfs;
use crate::include::fs::stat::S_IFBLK;
use crate::include::bits::errno::*;
use crate::fs::vfs::vfs_mount_root;
use crate::include::mm::kvmem::*;
use crate::include::core::types::devid_t;
use crate::{malloc_declare, print, DEV};

malloc_declare!(M_VNODE);

static mut rd_dev: *mut Vnode = core::ptr::null_mut();
static mut archivers: Queue<*mut Filesystem> = Queue::empty();

pub unsafe fn initramfs_archiver_register(fs: *mut Filesystem) -> isize {
    if archivers.enqueue(fs).is_null() {
        return -ENOMEM;
    }

    print!("initramfs: registered archiver: {}\n", (*fs).name);

    return 0;
}

pub unsafe fn load_ramdisk(_: *mut u8) -> isize {
    print!("kernel: loading ramdisk\n");

    rd_dev = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, M_ZERO) as *mut Vnode;

    if rd_dev.is_null() {
        return -ENOMEM;
    }

    (*rd_dev).mode = S_IFBLK;
    (*rd_dev).rdev = DEV!(1, 0);
    (*rd_dev).size = rd_size;
    (*rd_dev).fs   = &devfs;

    let mut root: *mut Vnode = core::ptr::null_mut();
    let mut err = -1;

    for node in archivers.iter() {
        let fs = (*node).value;
        err = (*fs).load(rd_dev, &mut root);

        if err == 0 {
            break;
        }
    }

    if err != 0{
        print!("error code = {}\n", err);
        panic!("could not load ramdisk\n");
    }

    return vfs_mount_root(root);
}

