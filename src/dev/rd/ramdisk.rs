use prelude::*;
use dev::dev::*;
use dev::kdev::*;
use boot::*;

extern "C" {
    static __kboot: *const BootInfo;
}

static mut RD_ADDR: *const u8 = core::ptr::null();

pub static mut RD_SIZE: usize = 0; /* XXX */

unsafe fn rd_read(_dd: *mut DeviceDescriptor, offset: isize, size: usize, buf: *mut u8) -> isize {
    /* maximum possible read size */
    let size = if size < RD_SIZE - offset as usize { size } else { RD_SIZE - offset as usize };
    
    /* copy `size' bytes from ramdev into buffer */
    memcpy(buf, RD_ADDR.offset(offset), size);

    return size as isize;
}

/*
unsafe extern "C" fn rd_write(_dd: *mut DeviceDescriptor, offset: isize, size: usize, buf: *mut u8) -> isize {
    /* maximum possible write size */
    let size = if size < rd_size - offset as usize { size } else { rd_size - offset as usize };
    
    /* copy `size' bytes from buffer to ramdev */
    memcpy(rd_addr.offset(offset), buf, size);

    return size as isize;
}
*/

unsafe fn rd_probe() -> isize {
    let rd_module = (*__kboot).modules.offset(0);
    RD_ADDR = (*rd_module).addr;
    RD_SIZE = (*rd_module).size;

    kdev_blkdev_register(1, &mut RDDEV);

    return 0;
}

unsafe fn rd_getbs(_dd: *mut DeviceDescriptor) -> usize {
    return 1;   /* FIXME */
}

static mut RDDEV: Device = Device {
    name:  "ramdisk",
    probe: Some(rd_probe),
    read:  Some(rd_read),
    //write: Some(rd_write),
    getbs: Some(rd_getbs),

    ..Device::none()
};

module_init!(rd, Some(rd_probe), None);
