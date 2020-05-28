use prelude::*;
use fs::*;
use bits::dirent::DirectoryEntry;
use net::socket::Socket;

#[derive(Copy, Clone)]
pub union FileBackend {
    pub vnode: *mut Vnode,
    pub socket: *mut Socket,
}

#[derive(Copy, Clone)]
pub struct FileDescriptor {
    pub backend: FileBackend,
    pub offset: off_t,
    pub flags: usize,
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

