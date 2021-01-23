use prelude::*;

use arch;
use arch::sys::sched::*;
use bits::dirent::*;
use bits::fcntl::*;
use bits::mman::*;
use bits::utsname::*;
use fs::{self, *};
use kern::time::*;
use mm::*;
use net::socket::*;
use sys::execve::*;
use sys::fork::*;
use sys::pgroup::*;
use sys::process::*;
use sys::sched::*;
use sys::session::*;
use sys::signal::*;
use sys::thread::*;

/* XXX */
const FDS_COUNT: usize = 64;

#[derive(Copy, Clone)]
pub union FileBackend {
    pub vnode: *mut Node,
    pub socket: *mut Socket,
}

#[derive(Copy, Clone)]
pub struct FileDescriptor {
    pub backend: FileBackend,
    pub offset: off_t,
    pub flags: usize,
}


impl FileDescriptor {
    pub fn open(&self) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_none() {
                return -ENOSYS;
            }

            match (*self.backend.vnode).fs.as_ref().unwrap().fops._open {
                Some(f) => f(self as *const _ as *mut FileDescriptor),
                None => -ENOSYS
            }
        }
    }

    pub fn read(&mut self, buf: *mut u8, size: usize) -> isize {
        unsafe {
            //fs::posix::posix_file_read(self as *const _ as *mut FileDescriptor, buf, size)
            //print!("posix::file_read(file={:?}, buf={:?}, size={})\n", file, buf, size);
            
            if self.flags & O_WRONLY != 0 {
                /* file is not opened for reading */
                return -EBADFD;
            }

            if size == 0 {
                return 0;
            }
            
            let mut retval = 0;

            loop {
                match (*self.backend.vnode).read(self.offset as usize, size, buf) {
                    Ok(retval) => {
                        if retval > 0 {
                            /* update file offset */
                            self.offset += retval as isize;
                            
                            /* wake up all sleeping writers if a `write_queue' is attached */
                            if !(*self.backend.vnode).write_queue.is_none() {
                                thread_queue_wakeup((*self.backend.vnode).write_queue.as_mut().unwrap().as_mut());
                            }

                            /* return read bytes count */
                            return retval as isize;
                        } else if vfs_file_eof(self) != 0 {
                            /* reached end-of-file */
                            return 0;
                        } else if self.flags & O_NONBLOCK != 0 {
                            /* can not satisfy read operation, would block */
                            return -EAGAIN;
                        } else {
                            /* block until some data is available */
                            /* sleep on the file readers queue */
                            if thread_queue_sleep((*self.backend.vnode).read_queue.as_mut().unwrap().as_mut()) != 0 {
                                return -EINTR;
                            }
                        }
                    },
                    Err(err) => {
                        return err.unwrap();
                    }
                }
            }
        }
    }

    pub fn write(&mut self, buf: *mut u8, size: usize) -> isize {
        unsafe {
            if self.flags & (O_WRONLY | O_RDWR) == 0 {
                /* file is not opened for writing */
                return -EBADFD;
            }
            
            if self.flags & O_NONBLOCK != 0 {
                /* non-blocking I/O */
                if vfs_file_can_write(self, size) != 0 {
                    /* write up to `size' from `buf' into file */
                    match (*self.backend.vnode).write(self.offset as usize, size, buf) {
                        Ok(retval) => {
                            /* update file offset */
                            self.offset += retval as isize;
                            
                            /* wake up all sleeping readers if a `read_queue' is attached */
                            if !(*self.backend.vnode).read_queue.is_none() {
                                thread_queue_wakeup((*self.backend.vnode).read_queue.as_mut().unwrap().as_mut());
                            }

                            /* return written bytes count */
                            return retval as isize;
                        },
                        Err(err) => {
                            return err.unwrap();
                        }
                    }

                } else {
                    /* can not satisfy write operation, would block */
                    return -EAGAIN;
                }
            } else {
                /* blocking I/O */
                let mut retval = size as isize;
                let mut size = size;
                
                while size > 0 {
                    match (*self.backend.vnode).write(self.offset as usize, size, buf) {
                        Ok(retval) => {
                            size -= retval;

                            /* no bytes left to be written, or reached end-of-file */
                            if size == 0 || vfs_file_eof(self) != 0 {
                                /* done writting */
                                break;
                            }

                            /* sleep on the file writers queue */
                            thread_queue_sleep((*self.backend.vnode).write_queue.as_mut().unwrap().as_mut());
                        },
                        Err(err) => {
                            return err.unwrap();
                        }
                    }
                }
                
                /* store written bytes count */
                retval -= size as isize;

                /* update file offset */
                self.offset += retval;

                /* wake up all sleeping readers if a `read_queue' is attached */
                if !(*self.backend.vnode).read_queue.is_none() {
                    thread_queue_wakeup((*self.backend.vnode).read_queue.as_mut().unwrap().as_mut());
                }

                return retval;
            }
        }
    }

    pub fn readdir(&mut self, dirent: *mut DirectoryEntry) -> isize {
        unsafe {
            if self.flags & O_WRONLY != 0 {
                /* file is not opened for reading */
                return -EBADFD;
            }
            
            match (*self.backend.vnode).readdir(self.offset as usize) {
                Ok((off, ent)) => {
                    /* update file offset */
                    self.offset += off as isize;
                        
                    dirent.write(ent);

                    /* return read bytes count */
                    return off as isize;
                },
                Err(err) => err.unwrap(),
            }
        }
    }

    pub fn lseek(&mut self, offset: off_t, whence: isize) -> off_t {
        const SEEK_SET: isize = 0;
        const SEEK_CUR: isize = 1;
        const SEEK_END: isize = 2;

        unsafe {
            let node = self.backend.vnode;

            match whence {
                SEEK_SET => self.offset = offset,
                SEEK_CUR => self.offset += offset,
                SEEK_END => self.offset = (*node).size() as isize + offset,
                _ => {}
            }

            return self.offset;
        }
    }

    pub fn close(&mut self) -> isize {
        unsafe {
            match fs::close(&*self.backend.vnode) {
                Ok(val) => val as isize,
                Err(err) => err.unwrap()
            }
        }
    }

    pub fn ioctl(&mut self, request: usize, argp: *mut u8) -> isize {
        unsafe {
            match fs::ioctl(&*self.backend.vnode, request, argp) {
                Ok(val) => val as isize,
                Err(err) => err.unwrap(),
            }
        }
    }

    pub fn trunc(&mut self, len: off_t) -> isize {
        unsafe {
            match (*self.backend.vnode).trunc(len as usize) {
                Ok(val) => 0,
                Err(err) => err.unwrap(),
            }
        }
    }

    /* helpers */
    pub fn can_read(&self, size: usize) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_none() {
                return -ENOSYS;
            }

            match (*self.backend.vnode).fs.as_ref().unwrap().fops._can_read {
                Some(f) => f(self as *const _ as *mut FileDescriptor, size),
                None => -ENOSYS
            }
        }
    }

    pub fn can_write(&self, size: usize) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_none() {
                return -ENOSYS;
            }

            match (*self.backend.vnode).fs.as_ref().unwrap().fops._can_write {
                Some(f) => f(self as *const _ as *mut FileDescriptor, size),
                None => -ENOSYS
            }
        }
    }

    pub fn eof(&self) -> isize {
        unsafe {
            if (*self.backend.vnode).fs.is_none() {
                return -ENOSYS;
            }

            match (*self.backend.vnode).fs.as_ref().unwrap().fops._eof {
                Some(f) => f(self as *const _ as *mut FileDescriptor),
                None => -ENOSYS
            }
        }
    }
}



pub unsafe fn open(path: *const u8, oflags: usize, mode: mode_t) {
    //syscall_log!(LOG_DEBUG, "open(path={}, oflags={:o}, mode={:o})\n", cstr(path), oflags, mode);

    /* look up the file */
    let mut uio = proc_uio!(curproc!());
    uio.flags = oflags;

    match fs::lookup(&cstr(path), &uio) {
        Err(err) => {
            /* lookup failed */
            if (err == ENOENT) && (oflags & O_CREAT) != 0 {
                match fs::creat(&cstr(path), mode, &mut uio) {
                    Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
                    Ok(_) => {
                        open(path, oflags, mode);
                        return;
                    }
                }
            } else {
                arch::syscall_return(curthread!(), err.unwrap() as usize);
                return;
            }
        },
        Ok((node, _)) => {
            let mut file = FileDescriptor {
                offset: 0,
                flags:  oflags,
                backend: FileBackend {
                    vnode: node,
                }
            };

            let err = vfs_perms_check(&mut file, &mut uio);
            if err != 0 {
                arch::syscall_return(curthread!(), err as usize);
                return;
            }

            if let Err(err) = vfs_file_open(&mut file) {
                arch::syscall_return(curthread!(), err.unwrap() as usize);
                return;
            }

            let fd = proc_fd_get(curproc!());
            if fd == -1 {
                /* reached maximum number of open file descriptors */
                arch::syscall_return(curthread!(), -EMFILE as usize);
                return;
            }

            (*curproc!()).fds.offset(fd).write(file);

            /* return the file descriptor */
            arch::syscall_return(curthread!(), fd as usize);
            return;
        }
    }
}


pub unsafe fn read(fildes: isize, buf: *mut u8, nbytes: size_t) {
    //syscall_log(LOG_DEBUG, "read(fd=%d, buf=%p, count=%d)\n", fildes, buf, nbytes);
    
    if fildes < 0 || (fildes as usize) >= FDS_COUNT {  /* Out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fildes);
    let err = vfs_file_read(file, buf, nbytes);
    arch::syscall_return(curthread!(), err as usize);
    return;
}

pub unsafe fn write(fd: isize, buf: *mut u8, nbytes: size_t) {
    //syscall_log!(LOG_DEBUG, "write(fd={}, buf={:p}, nbytes={})\n", fd, buf, nbytes);
    
    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fd);
    let err = vfs_file_write(file, buf, nbytes);
    arch::syscall_return(curthread!(), err as usize);
    return;
}


pub unsafe fn ioctl(fd: isize, request: isize, argp: *mut u8) {
    //syscall_log(LOG_DEBUG, "ioctl(fd=%d, request=0x%x, argp=%p)\n",
    //        fd, request, argp);

    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fd);
    let err = vfs_file_ioctl(file, request as usize, argp);
    arch::syscall_return(curthread!(), err as usize);
    return;
}

pub unsafe fn readdir(fd: isize, dirent: *mut DirectoryEntry) {
    //syscall_log(LOG_DEBUG, "readdir(fd=%d, dirent=%p)\n", fd, dirent);
    
    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fd);
    let err = vfs_file_readdir(file, dirent);
    arch::syscall_return(curthread!(), err as usize);
    return;
}

#[repr(C)]
pub struct MountArgs {
    fs_type: *const u8,
    dir: *const u8,
    flags: isize,
    data: *mut u8,
}


pub unsafe fn mount(args: *mut MountArgs) {
    if (*curproc!()).uid != 0 {
        arch::syscall_return(curthread!(), -EACCES as usize);
        return;
    }

    let fs_type = (*args).fs_type;
    let dir     = (*args).dir;
    let flags   = (*args).flags;
    let data    = (*args).data;

    match fs::mount(&cstr(fs_type), &cstr(dir), flags, data, &proc_uio!(curproc!())) {
        Ok(_) => arch::syscall_return(curthread!(), 0),
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
    }
}


pub unsafe fn mkdir(path: *const u8, mode: mode_t) {
    //syscall_log(LOG_DEBUG, "mkdir(path=%s, mode=%x)\n", path, mode);

    let mut uio = proc_uio!(curproc!());

    match fs::mkdir(&cstr(path), mode, &mut uio) {
        Ok(_) => arch::syscall_return(curthread!(), 0),
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
    }
}

pub unsafe fn pipe(fd: &mut [isize; 2]) {
    //syscall_log(LOG_DEBUG, "pipe(fd=%p)\n", fd);

    /*
    let fd1 = proc_fd_get(curproc!());
    let fd2 = proc_fd_get(curproc!());

    pipefs_pipe((*curproc!()).fds.offset(fd1), (*curproc!()).fds.offset(fd2));

    fd[0] = fd1;
    fd[1] = fd2;
    */

    arch::syscall_return(curthread!(), 0);
}


pub unsafe fn fcntl(fd: isize, cmd: isize, arg: usize) {
    //syscall_log(LOG_DEBUG, "fcntl(fd=%d, cmd=%d, arg=0x%x)\n",
    //        fd, cmd, arg);

    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fd);

    let mut dupfd = 0;

    match (cmd as usize) {
        F_DUPFD => {
            if arg == 0 {
                dupfd = proc_fd_get(curproc!());
            } else {
                dupfd = arg as isize;
            }

            *((*curproc!()).fds.offset(dupfd)) = *file;
            arch::syscall_return(curthread!(), dupfd as usize);
            return;
        }
        F_GETFD => {
            arch::syscall_return(curthread!(), (*file).flags);
            return;
        }
        F_SETFD => {
            (*file).flags = arg as usize; /* XXX */
            arch::syscall_return(curthread!(), 0);
            return;
        }
        _ => {
            arch::syscall_return(curthread!(), -EINVAL as usize);
            return;
        }
    }

    arch::syscall_return(curthread!(), -EINVAL as usize);
}


#[repr(C)]
pub struct MmapArgs {
    addr: usize,
    len: size_t,
    prot: usize,
    flags: usize,
    fildes: isize,
    off: off_t,
}


pub unsafe fn mmap(args: *mut MmapArgs, ret: *mut *mut u8) {
    //syscall_log(LOG_DEBUG, "mmap(addr=%p, len=%d, prot=%x, flags=%x, fildes=%d, off=%d, ret=%p)\n",
    //        args->addr, args->len, args->prot, args->flags, args->fildes, args->off, ret);

    let mut err = 0;

    let fildes = (*args).fildes;
    if fildes < 0 || (fildes as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fildes);

    if (*file).backend.vnode.is_null() {
        /* invalid file descriptor */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return;
    }

    let vm_entry = Box::leak(VmEntry::alloc(VmEntry::new()));

    /* initialize vm entry */
    vm_entry.base   = (*args).addr;
    vm_entry.size   = (*args).len;
    vm_entry.flags  = if (*args).prot & PROT_READ  != 0 { VM_UR } else { 0 };
    vm_entry.flags |= if (*args).prot & PROT_WRITE != 0 { VM_UW } else { 0 };
    vm_entry.flags |= if (*args).prot & PROT_EXEC  != 0 { VM_UX } else { 0 };
    vm_entry.flags |= if (*args).flags & MAP_SHARED != 0 { VM_SHARED } else { 0 };
    vm_entry.off    = (*args).off as usize;

    vm_entry.vm_object = vm_object_vnode((*file).backend.vnode);

    if (*args).flags & MAP_FIXED == 0 {
        /* allocate memory region */
        (*vm_entry).base = 0;
    }

    let vm_space = &mut (*curproc!()).vm_space;

    err = vm_space.insert(vm_entry);
    if err != 0 {
        if !vm_entry.qnode.is_null() {
            vm_space.vm_entries.node_remove(vm_entry.qnode);
        }

        Box::from_raw(vm_entry);

        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    if (*args).flags & MAP_PRIVATE == 0 {
        err = vfs_map(vm_space, vm_entry);

        if err != 0 {
            if !vm_entry.qnode.is_null() {
                vm_space.vm_entries.node_remove(vm_entry.qnode);
            }

            Box::from_raw(vm_entry);

            arch::syscall_return(curthread!(), err as usize);
            return;
        }
    }

    *ret = vm_entry.base as *mut u8;

    arch::syscall_return(curthread!(), err as usize);
    return;
}


pub unsafe fn munmap(addr: usize, len: size_t) {
    //syscall_log(LOG_DEBUG, "munmap(addr=%p, len=%d)\n", addr, len);

    let vm_space = &mut (*curproc!()).vm_space;

    for node in (*vm_space).vm_entries.iter() {
        let vm_entry = (*node).value as *mut VmEntry;

        if ((*vm_entry).base == addr && (*vm_entry).size == len) {
            (*vm_space).vm_entries.node_remove((*vm_entry).qnode);
            vm_unmap_full(vm_space, vm_entry);
            kfree(vm_entry as *mut u8);
            arch::syscall_return(curthread!(), 0);
            return;
        }
    }

    /* not found */
    arch::syscall_return(curthread!(), -EINVAL as usize);
    return;
}


pub unsafe fn close(fildes: isize) {
    //syscall_log!(LOG_DEBUG, "close(fildes={})\n", fildes);

    if fildes < 0 || (fildes as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fildes);
    
    if file == (-1isize as *mut _) || (*file).backend.vnode.is_null() {
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return;
    }

    let ret = vfs_file_close(file);
    (*(*curproc!()).fds.offset(fildes)).backend.vnode = core::ptr::null_mut();
    arch::syscall_return(curthread!(), ret as usize);
}

pub unsafe fn fstat(fildes: isize, statbuf: *mut Stat) {
    //syscall_log(LOG_DEBUG, "fstat(fildes=%d, buf=%p)\n", fildes, buf);

    if fildes < 0 || (fildes as usize) >= FDS_COUNT {  /* Out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fildes);
    
    if (*file).backend.vnode.is_null() {
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return;
    }

    let vnode = (*file).backend.vnode;

    match fs::stat(&*vnode) {
        Ok(stat) => {
            statbuf.write(stat);
            arch::syscall_return(curthread!(), 0);
        },
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
    }
}

pub unsafe fn isatty(fildes: isize) {
    //syscall_log(LOG_DEBUG, "isatty(fildes=%d)\n", fildes);

    if fildes < 0 || (fildes as usize >= FDS_COUNT) {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let node = (*(*curproc!()).fds.offset(fildes)).backend.vnode;

    if node.is_null() {
        /* invalid file descriptor */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return;
    }

    //arch::syscall_return(curthread, node->rdev & (136 << 8));
    arch::syscall_return(curthread!(), 1);
}

pub unsafe fn rmdir(path: *const u8) {
    //syscall_log(LOG_DEBUG, "rmdir(path=%s)\n", path);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}

pub unsafe fn chown(path: *const u8, uid: uid_t, gid: gid_t) {
    //syscall_log(LOG_DEBUG, "chown(path=%s, uid=%d, gid=%d)\n", path, uid, gid);

    match fs::lookup(&cstr(path), &proc_uio!(curproc!())) {
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
        Ok((node, _)) => {
            match node.chown(uid, gid) {
                Ok(val) => arch::syscall_return(curthread!(), 0),
                Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
            }
        }
    }
}


pub unsafe fn fchown(fd: isize, owner: uid_t, group: gid_t) {
    //syscall_log(LOG_DEBUG, "fchown(fd=%d, owner=%d, group=%d)\n", fd, owner, group);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}


pub unsafe fn lchown(path: *const u8, owner: uid_t, group: gid_t) {
    //syscall_log(LOG_DEBUG, "lchown(path=%s, owner=%d, group=%d)\n", path, owner, group);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}


pub unsafe fn utime(path: *const u8, times: *const utimbuf) {
    //syscall_log(LOG_DEBUG, "utime(path=%s, times=%p)\n", path, times);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}

pub unsafe fn chmod(path: *const u8, mode: mode_t) {
    //syscall_log(LOG_DEBUG, "chmod(path=%s, mode=%d)\n", path, mode);

    let mut uio = proc_uio!(curproc!());

    match fs::lookup(&cstr(path), &uio) {
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
        Ok((node, _)) => {
            match node.chmod(mode) {
                Ok(val)  => arch::syscall_return(curthread!(), 0),
                Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
            }
        }
    }
}

const F_OK: usize = 0;
const X_OK: usize = 1;
const W_OK: usize = 2;
const R_OK: usize = 4;

pub unsafe fn access(path: *const u8, mode: usize) {
    //syscall_log(LOG_DEBUG, "access(path=%s, mode=%d)\n", path, mode);

    let mut err = 0;

    /* look up the file */
    let mut uio = proc_uio!(curproc!());

    uio.flags |= if mode & R_OK != 0 { O_RDONLY } else { 0 };
    uio.flags |= if mode & W_OK != 0 { O_WRONLY } else { 0 };
    uio.flags |= if mode & X_OK != 0 { O_EXEC   } else { 0 };

    match fs::lookup(&cstr(path), &uio) {
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
        Ok((node, realpath)) => {
            let mut file = FileDescriptor {
                backend: FileBackend { vnode: &mut *node },
                offset: 0,
                flags: uio.flags,
            };

            err = fs::vfs_perms_check(&mut file, &mut uio);
            arch::syscall_return(curthread!(), err as usize);
        }
    }
}

pub unsafe fn stat(path: *const u8, statbuf: *mut Stat) {
    //syscall_log(LOG_DEBUG, "stat(path=%s, buf=%p)\n", path, buf);

    let mut err = 0;
    let mut uio = proc_uio!(curproc!());

    match fs::lookup(&cstr(path), &uio) {
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
        Ok((node, _)) => {
            match fs::stat(node) {
                Ok(buf) => {
                    statbuf.write(buf);
                    arch::syscall_return(curthread!(), 0);
                },
                Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
            }
        }
    }
}

pub unsafe fn lseek(fildes: isize, offset: off_t, whence: isize) {
    //syscall_log(LOG_DEBUG, "lseek(fildes=%d, offset=%d, whence=%d)\n",
    //        fildes, offset, whence);

    if fildes < 0 || (fildes as usize >= FDS_COUNT) {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fildes);

    if (*file).backend.vnode.is_null() {
        /* invalid file descriptor */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return;
    }

    let err = fs::vfs_file_lseek(file, offset, whence);
    arch::syscall_return(curthread!(), err as usize);
}

pub unsafe fn link(oldpath: *const u8, newpath: *const u8) {
    //syscall_log(LOG_DEBUG, "link(oldpath=%s, newpath=%s)\n",
    //        oldpath, newpath);

    /* TODO */

    arch::syscall_return(curthread!(), -ENOSYS as usize);
}

pub unsafe fn unlink(path: *const u8) {
    //syscall_log(LOG_DEBUG, "unlink(path=%p)\n", path);

    match fs::unlink(&cstr(path), &proc_uio!(curproc!())) {
        Ok(_) => arch::syscall_return(curthread!(), 0),
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
    }
}

pub unsafe fn mknod(path: *const u8, mode: mode_t, dev: dev_t) {
    //syscall_log(LOG_DEBUG, "mknod(path=%s, mode=%x, dev=%x)\n", path, mode, dev);

    match fs::mknod(&cstr(path), mode, dev, &proc_uio!(curproc!())) {
        Ok(_) => arch::syscall_return(curthread!(), 0),
        Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
    }
}


pub unsafe fn lstat(path: *const u8, statbuf: *mut Stat) {
    if path.is_null() || statbuf.is_null() {
        arch::syscall_return(curthread!(), -Error::EINVAL as usize);
        return;
    }

    let mut uio = proc_uio!(curproc!());
    uio.flags = O_NOFOLLOW;

    match fs::lookup(&cstr(path), &uio) {
        Err(err) => arch::syscall_return(curthread!(), err as usize),
        Ok((node, _)) => {
            match fs::stat(node) {
                Ok(buf) => {
                    statbuf.write(buf);
                    arch::syscall_return(curthread!(), 0);
                },
                Err(err) => arch::syscall_return(curthread!(), err.unwrap() as usize),
            }
        }
    }
}

