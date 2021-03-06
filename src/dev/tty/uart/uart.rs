use prelude::*;

use dev::dev::*;
use dev::tty::generic::*;
use dev::tty::tty::*;
use fs::*;
use fs::posix::*;
use sys::process::*;
use sys::sched::*;
use sys::thread::*;
use sys::syscall::file::{FileDescriptor, FileBackend};

#[repr(C)]
pub struct Uart {
    pub name: *const u8,

    pub _in:  *mut RingBuffer,
    pub _out: *mut RingBuffer,

    pub tty: *mut Tty,

    /* vnode associated with uart device */
    pub vnode: *mut Node,

    pub init:     Option<unsafe fn(u: *mut Uart)>,
    pub transmit: Option<unsafe fn(u: *mut Uart, c: u8) -> isize>,
    pub receive:  Option<unsafe fn(u: *mut Uart) -> u8>,
}

const UART_BUF: usize = 64;

/* registered devices */
static mut DEVICES: [*mut Uart; 192] = [core::ptr::null_mut(); 192];

/* called when data is received */
pub unsafe fn uart_recieve_handler(u: *mut Uart, size: usize) {
    let mut buf: [u8; UART_BUF] = core::mem::uninitialized();

    for i in 0..size {
        buf[i] = (*u).receive.unwrap()(u);
    }

    tty_master_write((*u).tty, size, buf.as_mut_ptr());  
    thread_queue_wakeup((*(*u).vnode).read_queue.as_mut().unwrap().as_mut());
}

/* called when data is ready to be transmitted */
pub unsafe fn uart_transmit_handler(u: *mut Uart, size: usize) {
    let len = (*(*u)._out).available();
    let len = if size < len { size } else { len };

    for i in 0..len {
        let mut c = 0u8;

        (*(*u)._out).read(1, &mut c);
        (*u).transmit.unwrap()(u, c);
    }

    thread_queue_wakeup((*(*u).vnode).write_queue.as_mut().unwrap().as_mut());
}

/* tty interface */
pub unsafe fn uart_master_write(tty: *mut Tty, size: usize, buf: *const u8) -> isize {
    let u = (*tty).p as *mut Uart;
    let s = (*(*u)._out).write(size, buf as *mut u8);
    /* XXX */
    uart_transmit_handler(u, s);
    return s as isize;
}

pub unsafe fn uart_slave_write(tty: *mut Tty, size: usize, buf: *const u8) -> isize {
    let u = (*tty).p as *mut Uart;
    return (*(*u)._in).write(size, buf as *mut u8) as isize;
}

pub unsafe fn uart_read(dd: *mut DeviceDescriptor, _offset: off_t, size: usize, buf: *mut u8) -> isize {
    let u = DEVICES[(*dd).minor as usize - 64];

    if u.is_null() {
        return -EIO;
    }

    return (*(*u)._in).read(size, buf) as isize;
}

pub unsafe fn uart_write(dd: *mut DeviceDescriptor, _offset: off_t, size: usize, buf: *mut u8) -> isize {
    let u = DEVICES[(*dd).minor as usize - 64];
    if u.is_null() {
        return -EIO;
    }

    return tty_slave_write((*u).tty, size, buf);
}

pub unsafe fn uart_ioctl(dd: *mut DeviceDescriptor, request: usize, argp: *mut u8) -> isize {
    let u = DEVICES[(*dd).minor as usize - 64];

    if u.is_null() {
        return -EIO;
    }

    return tty_ioctl((*u).tty, request as isize, argp) as isize;
}

pub unsafe fn uart_file_open(file: *mut FileDescriptor) -> isize {
    let id = (*(*file).backend.vnode).rdev() & 0xFF - 64;
    let u = DEVICES[id as usize];
    let mut err = 0;

    if !(*u).vnode.is_null() {
        /* already open */
        /* XXX */
        (*file).backend.vnode = (*u).vnode;
    } else {
        (*u).init.unwrap()(u);
        (*u).vnode = (*file).backend.vnode;
        /* TODO Error checking */
        (*u)._in = Box::leak(RingBuffer::alloc(RingBuffer::new(UART_BUF)));
        (*u)._out = Box::leak(RingBuffer::alloc(RingBuffer::new(UART_BUF)));
        tty_new(curproc!(), 0, Some(uart_master_write), Some(uart_slave_write), u as *mut u8, &mut (*u).tty);
        (*(*file).backend.vnode).read_queue  = Some(Queue::alloc(Queue::new()));
        (*(*file).backend.vnode).write_queue = Some(Queue::alloc(Queue::new()));
    }

    return 0;
}

pub unsafe fn uart_register(id: isize, u: *mut Uart) -> isize {
    let mut id = id;

    if id < 0 {
        /* allocated dynamically */
        for i in 0..192 {
            if DEVICES[i].is_null() {
                DEVICES[i] = u;
                id = i as isize;
                break;
            }
        }

        if id < 0 {
            /* failed */
            return -1;
        }
    }

    DEVICES[id as usize] = u;

    print!("uart: registered uart {}: {}\n", id, cstr((*u).name));
    return id;
}

#[no_mangle]
pub static mut uart: Device = Device {
    name: "uart",

    read:  Some(uart_read),
    write: Some(uart_write),
    ioctl: Some(uart_ioctl),

    fops: FileOps {
        _open:  Some(uart_file_open),

        _can_write: Some(__vfs_can_always),  /* XXX */
        _eof: Some(__vfs_eof_never),

        ..FileOps::none()
    },

    ..Device::none()
};

