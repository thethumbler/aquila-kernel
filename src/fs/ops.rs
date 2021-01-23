use prelude::*;
use fs::{self, *};

use dev::kdev::*;
use dev::*;
use bits::dirent::*;

pub fn close(node: &Node) -> Result<usize, Error> {
    Ok(0)
}

pub fn unlink(path: &str, uio: &UserOp) -> Result<(), Error> {
    let realpath = fs::realpath(path, uio)?;
    let tokens: Vec<&str> = realpath.split("/").collect();

    if tokens.len() == 0 {
        return Err(Error::EINVAL);
    }

    let dirname  = tokens[..tokens.len() - 1].join("/");
    let basename = tokens.last().unwrap();
    let (dir, _) = fs::lookup(&dirname, uio)?;

    dir.unlink(basename, uio)
}

pub fn mknod(path: &str, mode: mode_t, dev: dev_t, uio: &UserOp) -> Result<Arc<Node>, Error> {
    let realpath = fs::realpath(path, uio)?;
    let tokens: Vec<&str> = realpath.split("/").collect();

    if tokens.len() == 0 {
        return Err(Error::EINVAL);
    }

    let dirname  = tokens[..tokens.len() - 1].join("/");
    let basename = tokens.last().unwrap();
    let (dir, _) = fs::lookup(&dirname, uio)?;

    dir.mknod(basename, mode, dev, uio)
}

pub fn mkdir(path: &str, mode: mode_t, uio: &UserOp) -> Result<Arc<Node>, Error> {
    mknod(path, S_IFDIR | mode, 0, uio)
}

pub fn creat(path: &str, mode: mode_t, uio: &UserOp) -> Result<Arc<Node>, Error> {
    mknod(path, S_IFREG | mode, 0, uio)
}

/* sync the metadata and/or data associated with a filesystem */
pub fn fssync(super_node: *mut Node, mode: isize) -> isize {
    return -Error::ENOTSUP;
}

/* sync all metadata and/or data of all filesystems */
pub fn sync(mode: isize) -> isize {
    return -Error::ENOTSUP;
}