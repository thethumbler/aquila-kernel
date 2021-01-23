use prelude::*;

use fs::{self, *};
use mm::*;
use bits::dirent::*;
use bits::fcntl::*;

fn follow(node: &mut Node, uio: &UserOp) -> Result<(&'static mut Node, String), Error> {
    /* TODO enforce limit */

    let mut buffer = Buffer::new(1024);
    let size = node.read(0, 1024, buffer.as_ptr_mut())?;

    lookup(&alloc::str::from_utf8(&buffer[..size]).unwrap(), uio)
}

pub fn lookup(path: &str, uio: &UserOp) -> Result<(&'static mut Node, String), Error> {
    let realpath = fs::realpath(path, uio)?;
    let Path { node: root, path: relative_path } = fs::mountpoint(&realpath)?;

    let mut root: &mut Node = unsafe { &mut *(root as *const _ as *mut Node) };
    let mut node: &mut Node = unsafe { &mut *(root as *const _ as *mut Node) };

    for token in relative_path.split("/") {
        if token.is_empty() {
            continue;
        }

        let dirent = node.finddir(token)?;
        node = fs::iget(root, dirent.d_ino)?;
    }

    /* resolve symbolic links */
    if node.is_symlink() && (uio.flags as usize & O_NOFOLLOW == 0) {
        return follow(node, uio);
    }

    return Ok((node, realpath));
}