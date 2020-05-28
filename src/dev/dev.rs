use prelude::*;
use fs::*;
use mm::*;

/* device descriptor */
pub struct DeviceDescriptor {
    pub devtype: mode_t,
    pub major:   devid_t,
    pub minor:   devid_t,
}

/* device */
pub struct Device {
    pub name: &'static str,

    pub probe:  Option<unsafe fn() -> isize>,
    pub read:   Option<unsafe fn(dd: *mut DeviceDescriptor, offset: off_t, size: usize, buf: *mut u8) -> isize>,
    pub write:  Option<unsafe fn(dd: *mut DeviceDescriptor, offset: off_t, size: usize, buf: *mut u8) -> isize>,
    pub ioctl:  Option<unsafe fn(dd: *mut DeviceDescriptor, request: usize, argp: *mut u8) -> isize>,
    pub map:    Option<unsafe fn(dd: *mut DeviceDescriptor, vm_space: *mut AddressSpace, vm_entry: *mut VmEntry) -> isize>,

    pub fops: FileOps,

    /* device multiplexr */
    pub mux: Option<unsafe fn(dd: *mut DeviceDescriptor) -> *mut Device>,

    /* block size, for blkdev */
    pub getbs: Option<unsafe fn(dd: *mut DeviceDescriptor) -> usize>,
}

impl Device {
    pub const fn none() -> Device {
        Device {
            name:   "",
            probe:  None,
            read:   None,
            write:  None,
            ioctl:  None,
            map:    None,
            mux:    None,
            getbs:  None,
            fops:   FileOps::none(),
        }
    }
}

/* useful macros */
pub macro devid {
    ($major:expr, $minor:expr) => {
        ((($major as u16 & 0xFF) << 8) | ($minor as u16 & 0xFF)) as u16
    }
}

pub macro devid_major {
    ($dev:expr) => {
        (($dev >> 8) & 0xFF) as devid_t
    }
}

pub macro devid_minor {
    ($dev:expr) => {
        (($dev >> 0) & 0xFF) as devid_t
    }
}

pub macro vnode_dev {
    ($vnode:expr) => {
        DeviceDescriptor {
            devtype: (*$vnode).mode & S_IFMT,
            major: devid_major!((*$vnode).rdev),
            minor: devid_minor!((*$vnode).rdev),
        }
    }
}
