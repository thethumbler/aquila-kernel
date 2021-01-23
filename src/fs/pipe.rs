use prelude::*;

use fs::*;
use fs::posix::*;
use mm::*;
use bits::fcntl::*;
use sys::syscall::file::{FileDescriptor, FileBackend};

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

fn read(node: &mut Node, _offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
    unsafe {
        let pipe = node.p as *mut Pipe;
        return Ok((*(*pipe).ring).read(size, buffer));
    }
}

fn write(node: &mut Node, _offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
    unsafe {
        let pipe = node.p as *mut Pipe;
        return Ok((*(*pipe).ring).write(size, buffer));
    }
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
        //kfree(pipe as *mut u8);
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

        //kfree(pipe as *mut u8);
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

    (*read).backend.vnode = kmalloc(core::mem::size_of::<Node>(), &M_VNODE, 0) as *mut Node;
    if (*read).backend.vnode.is_null() {
        pipefs_pfree(pipe);
        return -ENOMEM;
    }

    (*write).backend.vnode = kmalloc(core::mem::size_of::<Node>(), &M_VNODE, 0) as *mut Node;
    if (*write).backend.vnode.is_null() {
        //kfree((*read).backend.vnode as *mut u8);
        pipefs_pfree(pipe);
        return -ENOMEM;
    }

    (*(*read).backend.vnode).read_queue = Some(Queue::alloc(Queue::new()));
    if (*(*read).backend.vnode).read_queue.is_none() {
        //kfree((*read).backend.vnode as *mut u8);
        //kfree((*write).backend.vnode as *mut u8);
        pipefs_pfree(pipe);
        return -ENOMEM;
    }

    // XXX
    //(*(*write).backend.vnode).write_queue = (*(*read).backend.vnode).read_queue;

    (*(*read).backend.vnode).fs  = Some(&PIPEFS);
    (*(*read).backend.vnode).p   = pipe as *mut u8;

    (*(*write).backend.vnode).fs = Some(&PIPEFS);
    (*(*write).backend.vnode).p  = pipe as *mut u8;
    (*write).flags       = O_WRONLY;

    return 0;
}

static PIPEFS: Filesystem = Filesystem {
    name: "pipefs",
    nodev: 0,

    read:  Some(read),
    write: Some(write),

    fops: FileOps {
        _can_read:  Some(pipefs_can_read),
        _can_write: Some(pipefs_can_write),
        _eof:       None, //__vfs_eof_never,

        ..FileOps::none()
    },

    ..Filesystem::none()
};
