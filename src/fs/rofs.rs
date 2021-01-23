use prelude::*;
use fs::*;

pub fn write(node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
    Err(Error::EROFS)
}

pub fn trunc(node: &Node, len: usize) -> Result<usize, Error> {
    Err(Error::EROFS)
}

pub fn mknod(dir: &Node, name: &str, mode: mode_t, dev: dev_t, uio: &UserOp) -> Result<Arc<Node>, Error> {
    Err(Error::EROFS)
}

pub fn unlink(dir: &Node, name: &str, uio: &UserOp) -> Result<(), Error> {
    Err(Error::EROFS)
}
