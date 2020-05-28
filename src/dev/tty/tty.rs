use prelude::*;

use dev::dev::*;
use dev::kdev::*;
use dev::tty::uart::uart::uart;
use fs::ioctl::*;
use fs::termios::*;
use sys::pgroup::*;
use sys::process::*;
use sys::sched::*;

pub type ttyio = Option<unsafe fn(tty: *mut Tty, size: usize, buf: *const u8) -> isize>;

#[repr(C)]
pub struct Tty {
    /** cooking buffer */
    pub cook: *mut u8,

    /** current position in cooking buffer */
    pub pos: usize,

    pub tios: Termios,
    pub ws: Winsize,

    /** associated device */
    pub dev: *mut Device,

    /** controlling process */
    pub proc: *mut Process,

    /** foreground process group */
    pub fg: *mut ProcessGroup,

    /* interface */

    /** specific handler private data */
    pub p: *mut u8,

    /** master write handelr */
    pub master_write: ttyio, //Option<fn(tty: *mut Tty, size: usize, buf: *mut u8) -> isize>,

    /** slave write handler */
    pub slave_write: ttyio, //Option<fn(tty: *mut Tty, size: usize, buf: *mut u8) -> isize>,
}

pub const TTY_BUF_SIZE: usize = 512;

unsafe fn sttydev_mux(dd: *mut DeviceDescriptor) -> *mut Device {
    if (*dd).minor < 64 {
        //return &vtty;
    } else {
        return &mut uart;
    }

    return core::ptr::null_mut();
}

unsafe fn ttydev_mux(dd: *mut DeviceDescriptor) -> *mut Device {
    match (*dd).minor {
        /* /dev/tty */
        0 => (*(*(*curproc!()).pgrp).session).ctty as *mut Device,
/*
        /* /dev/console */
        1 => &condev;
        /* /dev/ptmx */
        2 => return &ptmdev;
*/
        _ => core::ptr::null_mut()
    }
}

unsafe fn ttydev_probe() -> isize {
    kdev_chrdev_register(4, &mut sttydev);
    kdev_chrdev_register(5, &mut ttydev);
    return 0;
}

static mut sttydev: Device = Device {
    name: "sttydev",
    mux:  Some(sttydev_mux),

    ..Device::none()
};

static mut ttydev: Device = Device {
    name:  "ttydev",
    mux:   Some(ttydev_mux),

    ..Device::none()
};

module_init!(ttydev, Some(ttydev_probe), None);
