use prelude::*;
use fs::*;

use dev::kdev::*;

use include::fs::vfs::*;
use crate::include::core::types::*;
use crate::include::bits::errno::*;
use crate::include::bits::fcntl::*;
use crate::sys::thread::*;

/**
 * posix_file_write
 *
 * Writes up to `size' bytes from `buf' into a file.
 *
 * @file    File Descriptor for the function to operate on.
 * @buf     Buffer to read from.
 * @size    Number of bytes to write.
 * @returns written bytes on success, or error-code on failure.
 */

pub unsafe fn posix_file_write(file: *mut FileDescriptor, buf: *mut u8, size: size_t) -> ssize_t {
    if (*file).flags & (O_WRONLY | O_RDWR) == 0 {
        /* file is not opened for writing */
        return -EBADFD;
    }
    
    if (*file).flags & O_NONBLOCK != 0 {
        /* non-blocking I/O */
        if vfs_file_can_write(file, size) != 0 {
            /* write up to `size' from `buf' into file */
            let retval = vfs_write((*file).backend.vnode, (*file).offset, size, buf);

            /* update file offset */
            (*file).offset += retval;
            
            /* wake up all sleeping readers if a `read_queue' is attached */
            if !(*(*file).backend.vnode).read_queue.is_none() {
                thread_queue_wakeup((*(*file).backend.vnode).read_queue.as_mut().unwrap().as_mut());
            }

            /* return written bytes count */
            return retval;
        } else {
            /* can not satisfy write operation, would block */
            return -EAGAIN;
        }
    } else {
        /* blocking I/O */
        let mut retval = size as isize;
        let mut size = size;
        
        while size > 0 {
            size -= vfs_write((*file).backend.vnode, (*file).offset, size, buf) as usize;

            /* no bytes left to be written, or reached end-of-file */
            if size == 0 || vfs_file_eof(file) != 0 {
                /* done writting */
                break;
            }

            /* sleep on the file writers queue */
            thread_queue_sleep((*(*file).backend.vnode).write_queue.as_mut().unwrap().as_mut());
        }
        
        /* store written bytes count */
        retval -= size as isize;

        /* update file offset */
        (*file).offset += retval;

        /* wake up all sleeping readers if a `read_queue' is attached */
        if !(*(*file).backend.vnode).read_queue.is_none() {
            thread_queue_wakeup((*(*file).backend.vnode).read_queue.as_mut().unwrap().as_mut());
        }

        return retval;
    }
}
