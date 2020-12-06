use prelude::*;

use fs::*;
use fs::posix::*;
use mm::*;
use bits::fcntl::*;

use crate::{malloc_define, malloc_declare};

pub const PIPE_BUFLEN: usize = 1024;

/** unix pipe */
pub struct Pipe {
    /** readers reference count */
    pub r_ref: usize,

    /** writers reference count */
    pub w_ref: usize,

    /** ring buffer */
    pub ring: *mut RingBuffer,
}

malloc_define!(M_PIPE, "pipe", "pipe structure");
malloc_declare!(M_VNODE);

unsafe fn pipefs_read(vnode: *mut Vnode, _offset: off_t, size: size_t, buf: *mut u8) -> usize {
    let pipe = (*vnode).p as *mut Pipe;
    return (*(*pipe).ring).read(size, buf);
}

unsafe fn pipefs_write(vnode: *mut Vnode, _offset: off_t, size: size_t, buf: *mut u8) -> usize {
    let pipe = (*vnode).p as *mut Pipe;
    return (*(*pipe).ring).write(size, buf);
}

unsafe fn pipefs_can_read(file: *mut FileDescriptor, size: size_t) -> isize {
    let node = (*file).backend.vnode;
    let pipe = (*node).p as *mut Pipe;
    return (size <= (*(*pipe).ring).available()) as isize;
}

unsafe fn pipefs_can_write(file: *mut FileDescriptor, size: size_t) -> isize {
    let node = (*file).backend.vnode;
    let pipe = (*node).p as *mut Pipe;
    return (size >= (*(*pipe).ring).size() - (*(*pipe).ring).available()) as isize;
}

unsafe fn pipefs_mkpipe(pipe_ref: *mut *mut Pipe) -> isize {
    let mut pipe = kmalloc(core::mem::size_of::<Pipe>(), &M_PIPE, M_ZERO) as *mut Pipe;
    if pipe.is_null() {
        return -ENOMEM;
    }

    (*pipe).ring = Box::leak(RingBuffer::alloc(RingBuffer::new((PIPE_BUFLEN))));
    
    if (*pipe).ring.is_null() {
        kfree(pipe as *mut u8);
        return -ENOMEM;
    }

    if !pipe_ref.is_null() {
        *pipe_ref = pipe;
    }

    return 0;
}

unsafe fn pipefs_pfree(pipe: *mut Pipe) {
    if !pipe.is_null() {
        if !(*pipe).ring.is_null() {
            Box::from((*pipe).ring);
        }

        kfree(pipe as *mut u8);
    }
}

#[no_mangle]
pub unsafe fn pipefs_pipe(read: *mut FileDescriptor, write: *mut FileDescriptor) -> isize {
    let mut err = 0;
    let mut pipe = core::ptr::null_mut();

    if read.is_null() || write.is_null() {
        return -EINVAL;
    }

    err = pipefs_mkpipe(&pipe as *const _ as *mut *mut Pipe);
    if err != 0 {
        return err;
    }

    (*read).backend.vnode = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, 0) as *mut Vnode;
    if (*read).backend.vnode.is_null() {
        pipefs_pfree(pipe);
        return -ENOMEM;
    }

    (*write).backend.vnode = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, 0) as *mut Vnode;
    if (*write).backend.vnode.is_null() {
        kfree((*read).backend.vnode as *mut u8);
        pipefs_pfree(pipe);
        return -ENOMEM;
    }

    (*(*read).backend.vnode).read_queue = Some(Queue::alloc(Queue::new()));
    if (*(*read).backend.vnode).read_queue.is_none() {
        kfree((*read).backend.vnode as *mut u8);
        kfree((*write).backend.vnode as *mut u8);
        pipefs_pfree(pipe);
        return -ENOMEM;
    }

    // XXX
    //(*(*write).backend.vnode).write_queue = (*(*read).backend.vnode).read_queue;

    (*(*read).backend.vnode).fs  = &pipefs as *const _ as *mut Filesystem;
    (*(*read).backend.vnode).p   = pipe as *mut u8;

    (*(*write).backend.vnode).fs = &pipefs as *const _ as *mut Filesystem;
    (*(*write).backend.vnode).p  = pipe as *mut u8;
    (*write).flags       = O_WRONLY;

    return 0;
}

#[no_mangle]
pub static pipefs: Filesystem = Filesystem {
    name: "pipefs",
    nodev: 0,

    _init:  None,
    _load:  None,
    _mount: None,

    vops: VnodeOps {
        _read:  Some(pipefs_read),
        _write: Some(pipefs_write),

        _chmod:   None,
        _chown:   None,
        _close:   None,
        _finddir: None,
        _ioctl:   None,
        _map:     None,
        _readdir: None,
        _sync:    None,
        _trunc:   None,
        _vget:    None,
        _vmknod:  None,
        _vunlink: None,
        _vsync:   None,
    },

    fops: FileOps {
        _read:      Some(posix_file_read),
        _write:     Some(posix_file_write),
        _can_read:  Some(pipefs_can_read),
        _can_write: Some(pipefs_can_write),
        _eof:       None, //__vfs_eof_never,

        _open:    None,
        _readdir: None,
        _trunc:   None,
        _close:   None,
        _ioctl:   None,
        _lseek:   None,
    },
};
