use prelude::*;
use fs::*;
use bits::fcntl::*;
use sys::syscall::file::{FileDescriptor, FileBackend};

pub unsafe fn posix_file_open(file: *mut FileDescriptor) -> isize {
    if (*file).flags & O_TRUNC != 0 {
        return vfs_file_trunc(file, 0);
    }

    return 0;
}
