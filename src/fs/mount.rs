use prelude::*;

use fs::*;
use kern::print::cstr;

use kern::string::*;
use crate::include::fs::vfs::*;
use crate::include::mm::kvmem::*;
use crate::include::bits::errno::*;
use crate::{print, malloc_define};
use fs::vfs::registered_fs;

static mut mounts_queue: Queue<Mountpoint> = Queue::empty();
pub static mut mounts: *mut Queue<Mountpoint> = unsafe { &mut mounts_queue };

malloc_define!(M_MOUNTPOINT, "mountpoint\0", "mount point structure\0");

pub unsafe fn vfs_mount(fs_type: *const u8, dir: *const u8, flags: isize, data: *mut u8, uio: *mut UserOp) -> isize {
    let mut fs: *mut Filesystem = core::ptr::null_mut();

    /* look up filesystem */
    let mut entry = registered_fs;
    while !entry.is_null() {
        if (*entry).name == cstr(fs_type) {
            fs = (*entry).fs;
            break;
        }

        entry = (*entry).next;
    }

    if fs.is_null() {
        return -EINVAL;
    }

    /* directory path must be absolute */
    let mut err = 0;
    let mut _dir = core::ptr::null_mut();

    err = vfs_parse_path(dir, uio, &mut _dir);
    if err != 0 {
        if !_dir.is_null() {
            kfree(_dir);
        }

        return err;
    }

    err = (*fs).mount(_dir, flags, data);
    
    if err == 0 {
        let mut mp = kmalloc(core::mem::size_of::<Mountpoint>(), &M_MOUNTPOINT, 0) as *mut Mountpoint;

        if mp.is_null() {
            /* TODO */
        }

        struct S {
            dev: *mut u8,
            opt: *mut u8,
        };

        let args = data as *mut S;

        (*mp).dev = if !(*args).dev.is_null() { strdup((*args).dev) } else { b"none\0".as_ptr() as *mut u8 };
        (*mp).fs_type = strdup(fs_type);
        (*mp).path = strdup(_dir);
        (*mp).options = b"\0".as_ptr() as *mut u8;

        (*mounts).enqueue(mp);
    }

    kfree(_dir);
    return err;
}
