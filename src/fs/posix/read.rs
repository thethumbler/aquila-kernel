use prelude::*;
use fs::*;

use crate::include::core::types::*;
use crate::include::bits::errno::*;
use crate::include::bits::fcntl::*;
use crate::sys::thread::*;

/**
 *
 * Reads up to `size` bytes from a file to `buf`.
 *
 * Conforming to `IEEE Std 1003.1, 2013 Edition'
 * 
 * @param file    File Descriptor for the function to operate on.
 * @param buf     Buffer to write to.
 * @param size    Number of bytes to read.
 * @param returns read bytes on success, or error-code on failure.
 */

pub unsafe fn posix_file_read(file: *mut FileDescriptor, buf: *mut u8, size: size_t) -> ssize_t {
    if (*file).flags & O_WRONLY != 0 {
        /* file is not opened for reading */
        return -EBADFD;
    }

    if size == 0 {
        return 0;
    }
    
    let mut retval = 0;
    loop {
        retval = vfs_read((*file).backend.vnode, (*file).offset, size, buf);
        if retval > 0 {
            /* update file offset */
            (*file).offset += retval;
            
            /* wake up all sleeping writers if a `write_queue' is attached */
            if !(*(*file).backend.vnode).write_queue.is_null() {
                thread_queue_wakeup((*(*file).backend.vnode).write_queue);
            }

            /* return read bytes count */
            return retval;
        } else if retval < 0 {
            /* error */
            return retval;
        } else if vfs_file_eof(file) != 0 {
            /* reached end-of-file */
            return 0;
        } else if (*file).flags & O_NONBLOCK != 0 {
            /* can not satisfy read operation, would block */
            return -EAGAIN;
        } else {
            /* block until some data is available */
            /* sleep on the file readers queue */
            if thread_queue_sleep((*(*file).backend.vnode).read_queue) != 0 {
                return -EINTR;
            }
        }
    }
}
