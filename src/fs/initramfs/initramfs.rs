use prelude::*;

use dev::*;
use dev::rd::ramdisk::RD_SIZE;
use fs::*;
use fs::devfs::*;
use kern::string::*;
use mm::*;

malloc_declare!(M_VNODE);

static mut RD_DEV: *mut Vnode = core::ptr::null_mut();
static mut ARCHIVERS: Queue<*mut Filesystem> = Queue::empty();

pub unsafe fn initramfs_archiver_register(fs: *mut Filesystem) -> isize {
    if ARCHIVERS.enqueue(fs).is_null() {
        return -ENOMEM;
    }

    print!("initramfs: registered archiver: {}\n", (*fs).name);

    return 0;
}

pub unsafe fn load_ramdisk(_: *mut u8) -> isize {
    print!("kernel: loading ramdisk\n");

    RD_DEV = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, M_ZERO) as *mut Vnode;

    if RD_DEV.is_null() {
        return -ENOMEM;
    }

    (*RD_DEV).mode = S_IFBLK;
    (*RD_DEV).rdev = devid!(1, 0);
    (*RD_DEV).size = RD_SIZE;
    (*RD_DEV).fs   = &DEVFS;

    let mut root: *mut Vnode = core::ptr::null_mut();
    let mut err = -1;

    for node in ARCHIVERS.iter() {
        let fs = (*node).value;
        err = (*fs).load(RD_DEV, &mut root);

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

