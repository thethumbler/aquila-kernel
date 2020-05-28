use prelude::*;

use crate::include::core::string::*;
use crate::include::core::types::*;
use crate::include::bits::errno::*;
use crate::include::bits::dirent::*;
use crate::include::fs::vfs::*;
use sys::thread::Thread;

use crate::mm::*;

/** in-core inode structure (vnode) */
#[repr(C)]
pub struct Vnode {
    pub ino:    ino_t,
    pub size:   usize,
    pub dev:    dev_t,
    pub rdev:   dev_t,
    pub mode:   mode_t,
    pub uid:    uid_t,
    pub gid:    gid_t,
    pub nlink:  nlink_t,

    pub atime:  _time_t,
    pub mtime:  _time_t,
    pub ctime:  _time_t,

    /** filesystem implementation */
    pub fs: *const Filesystem,

    /** filesystem handler private data */
    pub p: *mut u8,

    /** number of processes referencing this vnode */
    pub refcnt: usize,

    pub read_queue: *mut Queue<*mut Thread>,
    pub write_queue: *mut Queue<*mut Thread>,

    /** virtual memory object associated with vnode */
    pub vm_object: *mut VmObject,
}

impl Vnode {
    pub fn read(&self, offset: off_t, size: usize, buf: *mut u8) -> usize {
        if self.fs.is_null() {
            return -ENOSYS as usize;
        }

        unsafe {
            match (*self.fs).vops._read {
                Some(f) => f(self as *const _ as *mut Vnode, offset, size, buf),
                None => -ENOSYS as usize
            }
        }
    }

    pub fn write(&self, offset: off_t, size: usize, buf: *mut u8) -> usize {
        if self.fs.is_null() {
            return -ENOSYS as usize;
        }

        unsafe {
            match (*self.fs).vops._write {
                Some(f) => f(self as *const _ as *mut Vnode, offset, size, buf),
                None => -ENOSYS as usize
            }
        }
    }

    pub fn ioctl(&self, request: isize, argp: *mut u8) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._ioctl {
                Some(f) => f(self as *const _ as *mut Vnode, request, argp),
                None => -ENOSYS
            }
        }
    }

    pub fn close(&self) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._close {
                Some(f) => f(self as *const _ as *mut Vnode),
                None => -ENOSYS
            }
        }
    }

    pub fn trunc(&self, len: off_t) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._trunc {
                Some(f) => f(self as *const _ as *mut Vnode, len),
                None => -ENOSYS
            }
        }
    }

    pub fn chmod(&self, mode: mode_t) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._chmod {
                Some(f) => f(self as *const _ as *mut Vnode, mode),
                None => -ENOSYS
            }
        }
    }

    pub fn chown(&self, owner: uid_t, group: gid_t) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._chown {
                Some(f) => f(self as *const _ as *mut Vnode, owner, group),
                None => -ENOSYS
            }
        }
    }

    pub fn readdir(&self, offset: off_t, dirent: *mut DirectoryEntry) -> usize {
        if self.fs.is_null() {
            return -ENOSYS as usize;
        }

        unsafe {
            match (*self.fs).vops._readdir {
                Some(f) => f(self as *const _ as *mut Vnode, offset, dirent),
                None => -ENOSYS as usize
            }
        }
    }

    pub fn finddir(&self, name: *const u8, dirent: *mut DirectoryEntry) -> isize {
        if self.fs.is_null() {
            return -ENOSYS as isize;
        }

        unsafe {
            match (*self.fs).vops._finddir {
                Some(f) => f(self as *const _ as *mut Vnode, name, dirent),
                None => -ENOSYS
            }
        }
    }

    pub fn vmknod(&self, filename: *const u8, mode: mode_t, dev: dev_t, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._vmknod {
                Some(f) => f(self as *const _ as *mut Vnode, filename, mode, dev, uio, vnode_ref),
                None => -ENOSYS
            }
        }
    }

    pub fn vunlink(&self, filename: *const u8, uio: *mut UserOp) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._vunlink {
                Some(f) => f(self as *const _ as *mut Vnode, filename, uio),
                None => -ENOSYS
            }
        }
    }

    pub fn vget(&self, ino: ino_t, vnode_ref: *mut *mut Vnode) -> isize {
        if self.fs.is_null() {
            return -ENOSYS as isize;
        }

        unsafe {
            match (*self.fs).vops._vget {
                Some(f) => f(self as *const _ as *mut Vnode, ino, vnode_ref),
                None => -ENOSYS
            }
        }
    }

    pub fn vsync(&self, mode: isize) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._vsync {
                Some(f) => f(self as *const _ as *mut Vnode, mode),
                None => -ENOSYS
            }
        }
    }

    pub fn sync(&self, mode: isize) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._sync {
                Some(f) => f(self as *const _ as *mut Vnode, mode),
                None => -ENOSYS
            }
        }
    }

    pub fn map(&self, vm_space: *mut AddressSpace, vm_entry: *mut VmEntry) -> isize {
        if self.fs.is_null() {
            return -ENOSYS;
        }

        unsafe {
            match (*self.fs).vops._map {
                Some(f) => f(vm_space, vm_entry),
                None => -ENOSYS
            }
        }
    }
}

