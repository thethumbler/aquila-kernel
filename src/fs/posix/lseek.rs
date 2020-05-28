use prelude::*;
use fs::*;

/**
 * posix_file_lseek
 *
 * Conforming to `IEEE Std 1003.1, 2013 Edition'
 *
 */

pub unsafe fn posix_file_lseek(file: *mut FileDescriptor, offset: off_t, whence: isize) -> ssize_t {
    let vnode = (*file).backend.vnode;

    match whence {
        SEEK_SET => (*file).offset = offset,
        SEEK_CUR => (*file).offset += offset,
        SEEK_END => (*file).offset = (*vnode).size as isize + offset,
    }

    return (*file).offset;
}
