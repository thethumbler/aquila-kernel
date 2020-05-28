use prelude::*;
use fs::*;

pub unsafe fn posix_file_close(file: *mut FileDescriptor) -> isize {
    vfs_close((*file).backend.vnode)
}
