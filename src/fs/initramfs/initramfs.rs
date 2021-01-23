use prelude::*;

use dev::*;
use dev::rd::ramdisk::RD_SIZE;
use fs::{self, *};
use fs::devfs::*;
use kern::string::*;
use mm::*;

malloc_declare!(M_VNODE);

static mut ARCHIVERS: Vec<&'static mut Filesystem> = Vec::new();

pub fn archiver_register(fs: &'static mut Filesystem) -> Result<(), Error> {
    unsafe {
        print!("initramfs: registered archiver: {}\n", fs.name);
        ARCHIVERS.push(fs);
        Ok(())
    }
}

pub fn load_ramdisk(_: *mut u8) -> Result<(), Error> {
    unsafe {
        print!("kernel: loading ramdisk\n");

        if let Some(devfs) = fs::get_fs_by_name("devfs") {
            let mut rd_dev = Node::none();

            rd_dev.set_mode(S_IFBLK);
            rd_dev.set_size(RD_SIZE);
            rd_dev.rdev = devid!(1, 0);
            rd_dev.fs   = Some(devfs);

            let rd_dev = Arc::new(rd_dev);

            for ar in ARCHIVERS.iter() {
                if let Ok(root) = ar.load(Arc::clone(&rd_dev)) {
                    return fs::bind("/", root);
                }
            }
        }

        panic!("could not load ramdisk\n");
    }
}

