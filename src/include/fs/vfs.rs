use prelude::*;

use crate::include::core::types::*;
use crate::include::bits::dirent::*;
use crate::include::bits::errno::*;
use crate::include::net::socket::*;
use crate::kern::print::cstr;

use crate::mm::*;
use crate::include::mm::mm::*;
use crate::include::mm::vm::*;
use crate::include::fs::stat::*;

use crate::fs::{Vnode};
use crate::{S_ISCHR, S_ISBLK, print};

#[repr(C)]
#[derive(Clone)]
pub struct FileOps {
    pub _open:    Option<unsafe fn(file: *mut FileDescriptor) -> isize>,
    pub _read:    Option<unsafe fn(file: *mut FileDescriptor, buf: *mut u8, size: usize) -> isize>,
    pub _write:   Option<unsafe fn(file: *mut FileDescriptor, buf: *mut u8, size: usize) -> isize>,
    pub _readdir: Option<unsafe fn(file: *mut FileDescriptor, dirent: *mut DirectoryEntry) -> isize>, 
    pub _lseek:   Option<unsafe fn(file: *mut FileDescriptor, offset: off_t, whence: isize) -> off_t>,
    pub _close:   Option<unsafe fn(file: *mut FileDescriptor) -> isize>,
    pub _ioctl:   Option<unsafe fn(file: *mut FileDescriptor, request: usize, argp: *mut u8) -> isize>,
    pub _trunc:   Option<unsafe fn(file: *mut FileDescriptor, len: off_t) -> isize>,

    /* helpers */
    pub _can_read:  Option<unsafe fn(file: *mut FileDescriptor, size: usize) -> isize>,
    pub _can_write: Option<unsafe fn(file: *mut FileDescriptor, size: usize) -> isize>,
    pub _eof:       Option<unsafe fn(file: *mut FileDescriptor) -> isize>,
}

impl FileOps {
    pub const fn none() -> FileOps {
        FileOps {
            _open: None,
            _read: None,
            _write: None,
            _readdir: None,
            _lseek: None,
            _close: None,
            _ioctl: None,
            _trunc: None,
            _can_read: None,
            _can_write: None,
            _eof: None,
        }
    }
}

impl FileDescriptor {
    pub fn open(&self) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._open {
                Some(f) => f(self as *const _ as *mut FileDescriptor),
                None => -ENOSYS
            }
        }
    }

    pub fn read(&self, buf: *mut u8, size: usize) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._read {
                Some(f) => f(self as *const _ as *mut FileDescriptor, buf, size),
                None => -ENOSYS
            }
        }
    }

    pub fn write(&self, buf: *mut u8, size: usize) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._write {
                Some(f) => f(self as *const _ as *mut FileDescriptor, buf, size),
                None => -ENOSYS
            }
        }
    }

    pub fn readdir(&self, dirent: *mut DirectoryEntry) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._readdir {
                Some(f) => f(self as *const _ as *mut FileDescriptor, dirent),
                None => -ENOSYS
            }
        }
    }

    pub fn lseek(&self, offset: off_t, whence: isize) -> off_t {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._lseek {
                Some(f) => f(self as *const _ as *mut FileDescriptor, offset, whence),
                None => -ENOSYS
            }
        }
    }

    pub fn close(&self) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._close {
                Some(f) => f(self as *const _ as *mut FileDescriptor),
                None => -ENOSYS
            }
        }
    }

    pub fn ioctl(&self, request: usize, argp: *mut u8) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._ioctl {
                Some(f) => f(self as *const _ as *mut FileDescriptor, request, argp),
                None => -ENOSYS
            }
        }
    }

    pub fn trunc(&self, len: off_t) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._trunc {
                Some(f) => f(self as *const _ as *mut FileDescriptor, len),
                None => -ENOSYS
            }
        }
    }

    /* helpers */
    pub fn can_read(&self, size: usize) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._can_read {
                Some(f) => f(self as *const _ as *mut FileDescriptor, size),
                None => -ENOSYS
            }
        }
    }

    pub fn can_write(&self, size: usize) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._can_write {
                Some(f) => f(self as *const _ as *mut FileDescriptor, size),
                None => -ENOSYS
            }
        }
    }

    pub fn eof(&self) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_null() {
                return -ENOSYS;
            }

            match (*(*self.backend.vnode).fs).fops._eof {
                Some(f) => f(self as *const _ as *mut FileDescriptor),
                None => -ENOSYS
            }
        }
    }
}

/*
 * \ingroup vfs
 * \brief user I/O operation
 */
#[repr(C)]
pub struct UserOp {
    /* root directory */
    pub root:   *mut u8,

    /* current working directory */
    pub cwd:    *mut u8,

    pub uid:    uid_t,
    pub gid:    gid_t,
    pub mask:   mode_t,
    pub flags:  usize,
}

/*
 * \ingroup vfs
 * \brief vnode operations
 */
#[repr(C)]
#[derive(Clone)]
pub struct VnodeOps {
    pub _read:    Option<unsafe fn(vnode: *mut Vnode, offset: off_t, size: usize, buf: *mut u8) -> usize>,
    pub _write:   Option<unsafe fn(vnode: *mut Vnode, offset: off_t, size: usize, buf: *mut u8) -> usize>,
    pub _ioctl:   Option<unsafe fn(vnode: *mut Vnode, request: isize, argp: *mut u8) -> isize>,
    pub _close:   Option<unsafe fn(vnode: *mut Vnode) -> isize>,
    pub _trunc:   Option<unsafe fn(vnode: *mut Vnode, len: off_t) -> isize>,
    pub _chmod:   Option<unsafe fn(vnode: *mut Vnode, mode: mode_t) -> isize>,
    pub _chown:   Option<unsafe fn(vnode: *mut Vnode, owner: uid_t, group: gid_t) -> isize>,

    pub _readdir: Option<unsafe fn(dir: *mut Vnode, offset: off_t, dirent: *mut DirectoryEntry) -> usize>,
    pub _finddir: Option<unsafe fn(dir: *mut Vnode, name: *const u8, dirent: *mut DirectoryEntry) -> isize>,

    pub _vmknod:  Option<unsafe fn(dir: *mut Vnode, filename: *const u8, mode: mode_t, dev: dev_t, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize>,
    pub _vunlink: Option<unsafe fn(dir: *mut Vnode, filename: *const u8, uio: *mut UserOp) -> isize>,

    pub _vget:    Option<unsafe fn(super_node: *mut Vnode, ino: ino_t, vnode_ref: *mut *mut Vnode) -> isize>,

    pub _vsync:   Option<unsafe fn(vnode: *mut Vnode, mode: isize) -> isize>,
    pub _sync:    Option<unsafe fn(super_node: *mut Vnode, mode: isize) -> isize>,

    pub _map:     Option<unsafe fn(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> isize>,
}

impl VnodeOps {
    pub const fn empty() -> VnodeOps {
        VnodeOps {
            _read: None,
            _write: None,
            _ioctl: None,
            _close: None,
            _trunc: None,
            _chmod: None,
            _chown: None,
            _readdir: None,
            _finddir: None,
            _vmknod: None,
            _vunlink: None,
            _vget: None,
            _vsync: None,
            _sync: None,
            _map: None,
        }
    }
}

// FIXME
#[repr(C)]
pub struct VfsPath {
    pub root: *mut Vnode,
    pub tokens: *mut *mut u8,
}

/** filesystem structure */
#[repr(C)]
pub struct Filesystem {
    pub name: &'static str,

    pub _init:  Option<unsafe fn() -> isize>,
    pub _load:  Option<unsafe fn(dev: *mut Vnode, super_node: *mut *mut Vnode) -> isize>,
    pub _mount: Option<unsafe fn(dir: *const u8, flags: isize, data: *mut u8) -> isize>,

    pub vops: VnodeOps,
    pub fops: FileOps,

    /* flags */
    pub nodev: isize,
}

unsafe impl Sync for Filesystem {}

impl Filesystem {
    pub fn init(&self) -> isize {
        unsafe {
            match self._init {
                Some(f) => f(),
                None => -ENOSYS
            }
        }
    }

    pub fn load(&self, dev: *mut Vnode, super_node: *mut *mut Vnode) -> isize {
        unsafe {
            match self._load {
                Some(f) => f(dev, super_node),
                None => -ENOSYS
            }
        }
    }

    pub fn mount(&self, dir: *const u8, flags: isize, data: *mut u8) -> isize {
        unsafe {
            match self._mount {
                Some(f) => f(dir, flags, data),
                None => -ENOSYS
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union FileBackend {
    pub vnode: *mut Vnode,
    pub socket: *mut Socket,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FileDescriptor {
    pub backend: FileBackend,
    pub offset: off_t,
    pub flags: usize,
}

/* list of registered filesystems */
#[repr(C)]
pub struct FilesystemList {
    /** filesystem name */
    pub name: &'static str,

    /** filesystem structure */
    pub fs: *mut Filesystem,

    /** next entry in the list */
    pub next: *mut FilesystemList,
}

pub unsafe fn __vfs_can_always(_f: *mut FileDescriptor, _s: usize) -> isize { 1 }
pub unsafe fn __vfs_can_never (_f: *mut FileDescriptor, _s: usize) -> isize { 0 }
pub unsafe fn __vfs_eof_always(_f: *mut FileDescriptor) -> isize { 1 }
pub unsafe fn __vfs_eof_never (_f: *mut FileDescriptor) -> isize { 0 }

// FIXME
#[macro_export]
macro_rules! ISDEV {
    ($vnode:expr) => {
        S_ISCHR!((*$vnode).mode) || S_ISBLK!((*$vnode).mode)
    }
}

/* XXX */
#[repr(C)]
pub struct Mountpoint {
    pub dev: *mut u8,
    pub path: *mut u8,
    pub fs_type: *mut u8,
    pub options: *mut u8,
}
