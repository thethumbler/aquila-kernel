use prelude::*;
use fs::*;

/**
 * posix_file_lseek
 *
 * Conforming to `IEEE Std 1003.1, 2013 Edition'
 *
 */

// XXX
pub const SEEK_SET: isize = 0;
pub const SEEK_CUR: isize = 1;
pub const SEEK_END: isize = 2;

pub unsafe fn posix_file_lseek(file: *mut FileDescriptor, offset: off_t, whence: isize) -> ssize_t {
    let vnode = (*file).backend.vnode;

    match whence {
        SEEK_SET => (*file).offset = offset,
        SEEK_CUR => (*file).offset += offset,
        SEEK_END => (*file).offset = (*vnode).size as isize + offset,
        _ => {}
    }

    return (*file).offset;
}
