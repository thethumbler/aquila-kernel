use prelude::*;

use crate::include::fs::vfs::FileDescriptor;
use crate::include::bits::errno::*;
use crate::include::core::types::*;
use crate::dev::dev::Device;
use crate::dev::dev::DeviceDescriptor;
use crate::include::fs::stat::*;
use crate::mm::vm_space::AddressSpace;
use crate::mm::vm_entry::VmEntry;
use crate::include::mm::kvmem::*;
use crate::kern::print::cstr;

use crate::{malloc_define, print, S_ISCHR};

malloc_define!(M_KDEV_BLK, b"kdev-blk\0", b"kdev block buffer\0");

macro max {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    }
}

macro min {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    }
}

static mut chrdev: [*mut Device; 256] = [core::ptr::null_mut(); 256];
static mut blkdev: [*mut Device; 256] = [core::ptr::null_mut(); 256];

#[inline]
unsafe fn kdev_get(dd: *mut DeviceDescriptor) -> *mut Device {
    let mut dev = core::ptr::null_mut();

    match (*dd).devtype {
        S_IFCHR => { dev = chrdev[(*dd).major as usize]; },
        S_IFBLK => { dev = blkdev[(*dd).major as usize]; },
        _ => {}
    };

    if !dev.is_null() && !(*dev).mux.is_none() {
        /* mulitplexed device? */
        return (*dev).mux.unwrap()(dd);
    }

    return dev;
}

pub unsafe fn kdev_close(_dd: *mut DeviceDescriptor) -> isize {
    return 0;
}

pub unsafe fn kdev_bread(dd: *mut DeviceDescriptor, offset: isize, size: usize, buf: *mut u8) -> isize {
    let dev = kdev_get(dd);
    let bs: isize = (*dev).getbs.unwrap()(dd) as isize;

    let mut ret = 0;
    let mut err = 0;

    let mut offset = offset;
    let mut size = size;
    let mut cbuf = buf;
    let mut bbuf: *mut u8 = core::ptr::null_mut();

    /* read up to block boundary */
    if offset % bs != 0 {
        if bbuf.is_null() {
            bbuf = kmalloc(bs as usize, &M_KDEV_BLK, 0);

            if bbuf.is_null() {
                return -ENOMEM;
            }
        }

        let start = min!(bs - offset % bs, size as isize);

        if start != 0 {
            err = (*dev).read.unwrap()(dd, offset/bs, 1, bbuf);
            if err < 0 {
                //goto error;
                panic!("todo");
            }

            memcpy(cbuf, bbuf.offset(offset % bs), start as usize);

            ret    += start;
            size   -= start as usize;
            cbuf    = cbuf.offset(start as isize);
            offset += start;
        }
    }

    /* read entire blocks */
    let count = size/bs as usize;

    if count != 0 {
        err = (*dev).read.unwrap()(dd, (offset/bs) as isize, count, cbuf);
        if err < 0 {
            //goto error;
            panic!("todo");
        }

        ret    += count as isize * bs;
        size   -= count * bs as usize;
        cbuf    = cbuf.offset(count as isize * bs);
        offset += count as isize * bs;
    }

    let end = size % bs as usize;

    if end != 0 {
        if bbuf.is_null() {
            bbuf = kmalloc(bs as usize, &M_KDEV_BLK, 0);

            if bbuf.is_null() {
                return -ENOMEM;
            }
        }

        err = (*dev).read.unwrap()(dd, offset/bs, 1, bbuf);
        if err < 0 {
            //goto error;
            panic!("todo");
        }

        memcpy(cbuf, bbuf, end);
        ret += end as isize;
    }

    if !bbuf.is_null() {
        kfree(bbuf);
    }

    return ret as isize;
}

pub unsafe fn kdev_bwrite(dd: *mut DeviceDescriptor, offset: isize, size: usize, buf: *mut u8) -> isize {
    let dev = kdev_get(dd);
    let bs = (*dev).getbs.unwrap()(dd);


    let mut ret = 0;
    let mut offset = offset as usize;
    let mut size = size;
    let mut cbuf = buf;
    let mut bbuf: *mut u8 = core::ptr::null_mut();

    if offset % bs != 0 {
        if bbuf.is_null() {
            bbuf = kmalloc(bs, &M_KDEV_BLK, 0);

            if bbuf.is_null() {
                return -ENOMEM;
            }
        }

        /* write up to block boundary */
        let start = min!(bs - offset % bs, size);

        if start != 0 {
            (*dev).read.unwrap()(dd, (offset/bs) as isize, 1, bbuf);
            memcpy(bbuf.offset((offset % bs) as isize), cbuf, start);
            (*dev).write.unwrap()(dd, (offset/bs) as isize, 1, bbuf);

            ret    += start;
            size   -= start;
            cbuf    = cbuf.offset(start as isize);
            offset += start;
        }
    }


    /* write entire blocks */
    let count = size/bs;

    if count != 0 {
        (*dev).write.unwrap()(dd, (offset/bs) as isize, count, cbuf);

        ret    += count * bs;
        size   -= count * bs;
        cbuf    = cbuf.offset((count * bs) as isize);
        offset += count * bs;
    }

    let end = size % bs;

    if end != 0 {
        if bbuf.is_null() {
            bbuf = kmalloc(bs, &M_KDEV_BLK, 0);

            if bbuf.is_null() {
                return -ENOMEM;
            }
        }

        (*dev).read.unwrap()(dd, (offset/bs) as isize, 1, bbuf);
        memcpy(bbuf, cbuf, end);
        (*dev).write.unwrap()(dd, (offset/bs) as isize, 1, bbuf);
        ret += end;
    }

    if !bbuf.is_null() {
        kfree(bbuf);
    }

    return ret as isize;
}

pub unsafe fn kdev_read(dd: *mut DeviceDescriptor, offset: isize, size: usize, buf: *mut u8) -> isize {
    let dev = kdev_get(dd);
    
    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).read.is_none() {
        return -ENXIO;
    }

    if S_ISCHR!((*dd).devtype) {
        return (*dev).read.unwrap()(dd, offset, size, buf);
    } else {
        return kdev_bread(dd, offset, size, buf);
    }
}

pub unsafe fn kdev_write(dd: *mut DeviceDescriptor, offset: isize, size: usize, buf: *mut u8) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).write.is_none() {
        return -ENXIO;
    }

    if S_ISCHR!((*dd).devtype) {
        return (*dev).write.unwrap()(dd, offset, size, buf);
    } else {
        return kdev_bwrite(dd, offset, size, buf);
    }
}

pub unsafe fn kdev_ioctl(dd: *mut DeviceDescriptor, request: isize, argp: *mut u8) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).ioctl.is_none() {
        return -ENXIO;
    }

    return (*dev).ioctl.unwrap()(dd, request as usize, argp);
}

pub unsafe fn kdev_map(dd: *mut DeviceDescriptor, vm_space: *mut AddressSpace, vm_entry: *mut VmEntry) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).map.is_none() {
        return -ENXIO;
    }

    return (*dev).map.unwrap()(dd, vm_space, vm_entry);
}

pub unsafe fn kdev_file_open(dd: *mut DeviceDescriptor, file: *mut FileDescriptor) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._open.is_none() {
        return -ENXIO;
    }

    return (*dev).fops._open.unwrap()(file);
}

pub unsafe fn kdev_file_read(dd: *mut DeviceDescriptor, file: *mut FileDescriptor, buf: *mut u8, size: usize) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._read.is_none() {
        return -ENXIO;
    }

    return (*dev).fops._read.unwrap()(file, buf, size);
}

pub unsafe fn kdev_file_write(dd: *mut DeviceDescriptor, file: *mut FileDescriptor, buf: *mut u8, size: usize) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._write.is_none() {
        return -ENXIO;
    }

    return (*dev).fops._write.unwrap()(file, buf, size);
}

pub unsafe fn kdev_file_lseek(dd: *mut DeviceDescriptor, file: *mut FileDescriptor, offset: isize, whence: isize) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._lseek.is_none() {
        return -ENXIO;
    }

    return (*dev).fops._lseek.unwrap()(file, offset, whence);
}

pub unsafe fn kdev_file_close(dd: *mut DeviceDescriptor, file: *mut FileDescriptor) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._close.is_none() {
        return -EINVAL;
    }

    return (*dev).fops._close.unwrap()(file);
}

pub unsafe fn kdev_file_ioctl(dd: *mut DeviceDescriptor, file: *mut FileDescriptor, request: isize, argp: *mut u8) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._ioctl.is_none() {
        return -ENXIO;
    }

    return (*dev).fops._ioctl.unwrap()(file, request as usize, argp);
}

pub unsafe fn kdev_file_can_read(dd: *mut DeviceDescriptor, file: *mut FileDescriptor, size: usize) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._can_read.is_none() {
        return -ENXIO;
    }

    return (*dev).fops._can_read.unwrap()(file, size);
}

pub unsafe fn kdev_file_can_write(dd: *mut DeviceDescriptor, file: *mut FileDescriptor, size: usize) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._can_write.is_none() {
        return -ENXIO;
    }

    return (*dev).fops._can_write.unwrap()(file, size);
}

pub unsafe fn kdev_file_eof(dd: *mut DeviceDescriptor, file: *mut FileDescriptor) -> isize {
    let dev = kdev_get(dd);

    if dev.is_null() {
        return -ENXIO;
    }

    if (*dev).fops._eof.is_none() {
        return -ENXIO;
    }

    return (*dev).fops._eof.unwrap()(file);
}

/**
 * \ingroup kdev
 * \brief register a new character device
 */
pub unsafe fn kdev_chrdev_register(major: devid_t, dev: *mut Device) {
    chrdev[major as usize] = dev; /* XXX */
    print!("kdev: registered chrdev {}: {}\n", major, (*dev).name);
}

/**
 * \ingroup kdev
 * \brief register a new block device
 */
pub unsafe fn kdev_blkdev_register(major: devid_t, dev: *mut Device) {
    blkdev[major as usize] = dev; /* XXX */
    print!("kdev: registered blkdev {}: {}\n", major, (*dev).name);
}

/**
 * \ingroup kdev
 * \brief initialize kdev subsystem
 */
pub unsafe fn kdev_init() {
    print!("kdev: initializing\n");
}

