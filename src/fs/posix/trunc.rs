use prelude::*;
use fs::*;
use include::fs::vfs::*;

pub unsafe fn posix_file_trunc(file: *mut FileDescriptor, len: off_t) -> isize {
    return vfs_trunc((*file).backend.vnode, len);
}
