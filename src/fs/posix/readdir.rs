use prelude::*;
use fs::*;
use bits::fcntl::*;
use bits::dirent::*;

/**
 * posix_file_readdir
 *
 * Conforming to `IEEE Std 1003.1, 2013 Edition'
 * 
 * @file    File Descriptor for the function to operate on.
 * @dirent  Buffer to write to.
 */

pub unsafe fn posix_file_readdir(file: *mut FileDescriptor, dirent: *mut DirectoryEntry) -> ssize_t {
    if (*file).flags & O_WRONLY != 0 {
        /* file is not opened for reading */
        return -EBADFD;
    }
    
    let retval = vfs_readdir((*file).backend.vnode, (*file).offset, dirent);

    /* update file offset */
    (*file).offset += retval;
        
    /* return read bytes count */
    return retval;
}
