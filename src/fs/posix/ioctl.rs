use prelude::*;
use fs::*;

/**
 * posix_file_ioctl
 *
 * Conforming to `IEEE Std 1003.1, 2013 Edition'
 *
 */

pub unsafe fn posix_file_ioctl(file: *mut FileDescriptor, request: usize, argp: *mut u8) -> isize {
    return vfs_ioctl((*file).backend.vnode, request, argp);
}
