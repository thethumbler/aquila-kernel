use prelude::*;
use fs::{self, *};
use fs::tmpfs::*;
use mm::*;
use kern::time::*;

use crate::{malloc_declare};

malloc_declare!(M_VNODE);

/* devfs root directory (usually mounted on '/dev') */
pub static mut DEVFS_ROOT: Option<Arc<Node>> = None;

fn init() -> Result<(), Error> {
    unsafe {
        if let Some(tmpfs) = fs::get_fs_by_name("tmpfs") {
            /* devfs is really just tmpfs */
            let mut devfs = (*tmpfs).clone();
            devfs.name = "devfs";
            devfs.init = Some(init);
            devfs.mount = Some(mount);

            let devfs = Arc::new(devfs);
            let ts = gettime()?;

            let mut devfs_root = Node::none();

            devfs_root.set_mode(S_IFDIR | 0o775);
            devfs_root.set_nlink(2);

            devfs_root.fs     = Some(Arc::clone(&devfs));
            devfs_root.refcnt = 1;

            devfs_root.ctime = ts;
            devfs_root.atime = ts;
            devfs_root.mtime = ts;

            DEVFS_ROOT = Some(Arc::new(devfs_root));

            fs::install(devfs)
        } else {
            Err(Error::EINVAL)
        }
    }
}

fn mount(_fs: Arc<Filesystem>, dir: &str, flags: isize, data: *mut u8) -> Result<(), Error> {
    unsafe {
        if DEVFS_ROOT.is_none() {
            return Err(Error::EINVAL);
        }

        fs::bind(dir, Arc::clone(DEVFS_ROOT.as_ref().unwrap()))
    }
}

module_define!{
    "devfs",
    Some(|| { vec!["tmpfs"] }),
    Some(init),
    None
}
