use prelude::*;
use fs::*;
use bits::fcntl::*;
use bits::dirent::*;

use net::socket::*;
use dev::dev::*;
use dev::kdev::*;
use net::socket::*;

use sys::syscall::file::{FileDescriptor, FileBackend};

#[derive(Clone)]
pub struct FileOps {
    pub _open:      Option<unsafe fn(file: *mut FileDescriptor) -> isize>,
    //pub _close:     Option<unsafe fn(file: *mut FileDescriptor) -> isize>,

    /* helpers */
    pub _can_read:  Option<unsafe fn(file: *mut FileDescriptor, size: usize) -> isize>,
    pub _can_write: Option<unsafe fn(file: *mut FileDescriptor, size: usize) -> isize>,
    pub _eof:       Option<unsafe fn(file: *mut FileDescriptor) -> isize>,
}

impl FileOps {
    pub const fn none() -> FileOps {
        FileOps {
            _open:      None,
            //_close:     None,
            _can_read:  None,
            _can_write: None,
            _eof:       None,
        }
    }
}

pub fn vfs_file_open(file: &mut FileDescriptor) -> Result<usize, Error> {
    unsafe {
        if file.backend.vnode.is_null() || (*file.backend.vnode).fs.is_none() {
            return Err(Error::EINVAL);
        }

        if (*file.backend.vnode).is_directory() && (file.flags & O_SEARCH) == 0 {
            return Err(Error::EISDIR);
        }

        if (*file.backend.vnode).is_device() {
            return Error::wrap_isize_to_usize(kdev_file_open(&mut vnode_dev!(file.backend.vnode), file));
        }

        //if ((*(*(*file).backend.vnode).fs).fops.open as *const u8).is_null() {
        //    return -ENOSYS;
        //}

        return Error::wrap_isize_to_usize(file.open());
    }
}

/*
 * \ingroup vfs
 * \brief read from an open file
 */
pub unsafe fn vfs_file_read(file: *mut FileDescriptor, buf: *mut u8, nbytes: usize) -> isize {
    if !file.is_null() && ((*file).flags & FILE_SOCKET != 0) {
        return socket_recv(file, buf, nbytes, 0);
    }

    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    //if (*(*file).backend.vnode).is_device() {
    //    return kdev_file_read(&mut vnode_dev!((*file).backend.vnode), file, buf, nbytes);
    //}

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.read as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).read(buf, nbytes);
}

/*
 * \ingroup vfs
 * \brief write to an open file
 */
pub unsafe fn vfs_file_write(file: *mut FileDescriptor, buf: *mut u8, nbytes: usize) -> isize {
    if !file.is_null() && ((*file).flags & FILE_SOCKET) != 0 {
        return socket_send(file, buf, nbytes, 0);
    }

    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    //if (*(*file).backend.vnode).is_device() {
    //    return kdev_file_write(&mut vnode_dev!((*file).backend.vnode), file, buf, nbytes);
    //}

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.write as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).write(buf, nbytes);
}

/*
 * \ingroup vfs
 * \brief perform ioctl on an open file
 */
pub unsafe fn vfs_file_ioctl(file: *mut FileDescriptor, request: usize, argp: *mut u8) -> isize {
    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    //if (*(*file).backend.vnode).is_device() {
    //    return kdev_file_ioctl(&mut vnode_dev!((*file).backend.vnode), file, request as isize, argp);
    //}

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.ioctl as *mut u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).ioctl(request, argp);
}

/*
 * \ingroup vfs
 * \brief perform a seek in an open file
 */
pub unsafe fn vfs_file_lseek(file: *mut FileDescriptor, offset: off_t, whence: isize) -> off_t {
    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    //if (*(*file).backend.vnode).is_device() {
    //    return kdev_file_lseek(&mut vnode_dev!((*file).backend.vnode), file, offset, whence);
    //}

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.lseek as *mut u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).lseek(offset, whence);
}

/*
 * \ingroup vfs
 * \brief read entries from an open directory
 */
pub unsafe fn vfs_file_readdir(file: *mut FileDescriptor, dirent: *mut DirectoryEntry) -> isize {
    if file.is_null() || (*file).backend.vnode.is_null() || (*(*file).backend.vnode).fs.is_none() {
        return -EINVAL;
    }

    if !(*(*file).backend.vnode).is_directory() {
        return -ENOTDIR;
    }

    //if ((*(*(*file).backend.vnode).fs).fops.readdir as *mut u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).readdir(dirent);
}

/*
 * \ingroup vfs
 * \brief close an open file
 */
pub unsafe fn vfs_file_close(file: *mut FileDescriptor) -> isize {
    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    if (*file).flags & FILE_SOCKET != 0 {
        return socket_shutdown(file, SHUT_RDWR as isize);
    }

    if (*(*file).backend.vnode).is_device() {
        return kdev_file_close(&mut vnode_dev!((*file).backend.vnode), file);
    }

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.close as *mut u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).close();
}

/*
 * \ingroup vfs
 * \brief truncate an open file
 */
pub unsafe fn vfs_file_trunc(file: *mut FileDescriptor, len: off_t) -> isize {
    if file.is_null() && ((*file).flags & FILE_SOCKET) != 0 {
        return -EINVAL;
    }

    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    if (*(*file).backend.vnode).is_device() {
        return -EINVAL;
    }

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.trunc as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).trunc(len);
}

pub unsafe fn vfs_file_can_read(file: *mut FileDescriptor, size: usize) -> isize {
    if !file.is_null() && ((*file).flags & FILE_SOCKET != 0) {
        return socket_can_read(file, size);
    }

    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    if (*(*file).backend.vnode).is_device() {
        return kdev_file_can_read(&mut vnode_dev!((*file).backend.vnode), file, size);
    }

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.can_read as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).can_read(size);
}

pub unsafe fn vfs_file_can_write(file: *mut FileDescriptor, size: usize) -> isize {
    if !file.is_null() && ((*file).flags & FILE_SOCKET != 0) {
        return socket_can_write(file, size);
    }

    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    if (*(*file).backend.vnode).is_device() {
        return kdev_file_can_write(&mut vnode_dev!((*file).backend.vnode), file, size);
    }

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.can_write as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).can_write(size);
}

pub unsafe fn vfs_file_eof(file: *mut FileDescriptor) -> isize {
    if file.is_null() || (*file).backend.vnode.is_null() {
        return -EINVAL;
    }

    if (*(*file).backend.vnode).is_device() {
        return kdev_file_eof(&mut vnode_dev!((*file).backend.vnode), file);
    }

    //if (*(*file).backend.vnode).fs.is_null() {
    //    return -EINVAL;
    //}

    //if ((*(*(*file).backend.vnode).fs).fops.eof as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*file).eof();
}
