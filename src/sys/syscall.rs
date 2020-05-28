use prelude::*;

use arch;
use arch::sys::sched::*;
use bits::dirent::*;
use bits::fcntl::*;
use bits::mman::*;
use bits::utsname::*;
use fs::*;
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

unsafe fn sys_exit(code: isize) {
    //syscall_log(LOG_DEBUG, "exit(code=%d)\n", code);

    let owner = curproc!();

    (*owner).exit = proc_exit!(code, 0);  /* Child exited normally */

    proc_kill(owner);
    arch_sleep();

    /* we should never reach this anyway */
    loop {}
}

unsafe fn sys_close(fildes: isize) {
    //syscall_log(LOG_DEBUG, "close(fildes=%d)\n", fildes);

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

// FIXME argp and envp should be slices/arrays

unsafe fn sys_execve(path: *const u8, argp: *const *const u8, envp: *const *const u8) {
    //syscall_log(LOG_DEBUG, "execve(path=%s, argp=%p, envp=%p)\n", path, argp, envp);

    if path.is_null() || *path == 0 {
        arch::syscall_return(curthread!(), -ENOENT as usize);
        return;
    }

    let mut err = 0;
    let path = strdup(path);

    if path.is_null() {
        arch::syscall_return(curthread!(), -ENOMEM as usize);
        return;
    }

    err = proc_execve(curthread!(), path, argp, envp);

    kfree(path);

    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
    } else {
        sched_thread_spawn(curthread!());
    }
}


unsafe fn sys_fork() {
    //syscall_log(LOG_DEBUG, "fork()\n");

    let mut fork = core::ptr::null_mut();
    proc_fork(curthread!(), &mut fork);

    /* Returns are handled inside proc_fork */
    if !fork.is_null() {
        let thread = (*(*fork).threads.head).value as *mut Thread;
        sched_thread_ready(thread);
    }
}


unsafe fn sys_fstat(fildes: isize, buf: *mut Stat) {
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

    let err = vfs_stat(vnode, buf);
    arch::syscall_return(curthread!(), err as usize);
}


unsafe fn sys_getpid() {
    //syscall_log(LOG_DEBUG, "getpid()\n");
    arch::syscall_return(curthread!(), (*curproc!()).pid as usize);
}


unsafe fn sys_isatty(fildes: isize) {
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


unsafe fn sys_kill(pid: pid_t, sig: isize) {
    //syscall_log(LOG_DEBUG, "kill(pid=%d, sig=%d)\n", pid, sig);
    let err = signal_send(pid, sig);
    arch::syscall_return(curthread!(), err as usize);
}


unsafe fn sys_link(oldpath: *const u8, newpath: *const u8) {
    //syscall_log(LOG_DEBUG, "link(oldpath=%s, newpath=%s)\n",
    //        oldpath, newpath);

    /* TODO */

    arch::syscall_return(curthread!(), -ENOSYS as usize);
}


unsafe fn sys_lseek(fildes: isize, offset: off_t, whence: isize) {
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

    let err = vfs_file_lseek(file, offset, whence);
    arch::syscall_return(curthread!(), err as usize);
}


unsafe fn sys_open(path: *const u8, oflags: usize, mode: mode_t) {
    //syscall_log(LOG_DEBUG, "open(path=%s, oflags=0x%x, mode=0x%x)\n", path, oflags, mode);
    //print!("open(path={}, oflags={:#x}, mode={:#x})\n", cstr(path), oflags, mode);
    
    /* find a free file descriptor */
    let fd = proc_fd_get(curproc!());

    if fd == -1 {
        /* reached maximum number of open file descriptors */
        arch::syscall_return(curthread!(), -EMFILE as usize);
        return;
    }

    /* look up the file */
    let mut vnode = core::ptr::null_mut();
    let mut uio = proc_uio!(curproc!());
    uio.flags = oflags;

    let mut err = vfs_lookup(path, &mut uio, &mut vnode, core::ptr::null_mut());

    if err != 0 {
        /* lookup failed */
        if (err == -ENOENT) && (oflags & O_CREAT) != 0 {
            err = vfs_creat(path, mode, &mut uio, &mut vnode);
            if err != 0 {
                proc_fd_release(curproc!(), fd);
                arch::syscall_return(curthread!(), err as usize);
                return;
            }

            err = 0;
        }
    }

    if err != 0 {
        proc_fd_release(curproc!(), fd);
        vfs_close(vnode);
        arch::syscall_return(curthread!(), err as usize);
        return;
    }


    *(*curproc!()).fds.offset(fd) = FileDescriptor {
        offset: 0,
        flags:  oflags,
        backend: FileBackend {
            vnode: vnode,
        }
    };

    err = vfs_perms_check((*curproc!()).fds.offset(fd), &mut uio);
    if err != 0 {
        proc_fd_release(curproc!(), fd);
        vfs_close(vnode);
        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    err = vfs_file_open((*curproc!()).fds.offset(fd));
    if err != 0 {
        proc_fd_release(curproc!(), fd);
        vfs_close(vnode);
        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    /* return the file descriptor */
    arch::syscall_return(curthread!(), fd as usize);
    return;
}


unsafe fn sys_read(fildes: isize, buf: *mut u8, nbytes: size_t) {
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


unsafe fn sys_sbrk(incr: isize) {
    //syscall_log(LOG_DEBUG, "sbrk(incr=0x%x)\n", incr);

    let heap_start = (*curproc!()).heap_start;
    let heap       = (*curproc!()).heap;

    (*curproc!()).heap = heap + incr as usize;
    (*(*curproc!()).heap_vm).size = page_round!(heap + (incr as usize) - heap_start);

    arch::syscall_return(curthread!(), heap);
    return;
}


unsafe fn sys_stat(path: *const u8, buf: *mut Stat) {
    //syscall_log(LOG_DEBUG, "stat(path=%s, buf=%p)\n", path, buf);

    let mut vnode = core::ptr::null_mut();
    let mut err = 0;
    let mut uio = proc_uio!(curproc!());

    err = vfs_lookup(path, &mut uio, &mut vnode, core::ptr::null_mut());

    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    err = vfs_stat(vnode, buf);
    arch::syscall_return(curthread!(), err as usize);
}


unsafe fn sys_times() {
    /* TODO */
}


unsafe fn sys_unlink(path: *const u8) {
    //syscall_log(LOG_DEBUG, "unlink(path=%p)\n", path);

    let mut uio = proc_uio!(curproc!());
    let mut err = vfs_unlink(path as *mut u8, &mut uio);
    arch::syscall_return(curthread!(), err as usize);
}

/* FIXME: move this */
const WNOHANG: usize = 1;

unsafe fn sys_waitpid(pid: pid_t, stat_loc: *mut isize, options: usize) {
    //syscall_log(LOG_DEBUG, "waitpid(pid=%d, stat_loc=%p, options=0x%x)\n", pid, stat_loc, options);

    let nohang = (options & WNOHANG) != 0;

    if pid < -1 {
        /* wait for any child process whose process group ID
              is equal to the absolute value of pid */
        panic!("unsupported");
    } else if pid == -1 {

        /* wait for any child process */
        loop {
            let mut found = 0;

            for node in PROCS.iter() {
                let proc = (*node).value;

                if (*proc).parent != curproc!() {
                    continue;
                }

                found = 1;

                if (*proc).running == 0 {
                    if !stat_loc.is_null() {
                        *stat_loc = (*proc).exit;
                    }

                    arch::syscall_return(curthread!(), (*proc).pid as usize);
                    proc_reap(proc);
                    return;
                }
            }

            if nohang {
                arch::syscall_return(curthread!(), if found != 0 { 0 } else { -ECHILD as usize });
                return;
            }

            if thread_queue_sleep(&mut (*curproc!()).wait_queue) != 0 {
                arch::syscall_return(curthread!(), -EINTR as usize);
                return;
            }
        }

    } else if pid == 0 {
        /* wait for any child process whose process group ID
              is equal to that of the calling process */
        panic!("unsupported");
    } else {
        /* wait for the child whose process ID is equal to the
              value of pid */
        let child = proc_pid_find(pid);

        /* If pid is invalid or current process is not parent of child */
        if child.is_null() || (*child).parent != curproc!() {
            arch::syscall_return(curthread!(), -ECHILD as usize);
            return;
        }

        if (*child).running == 0 {
            /* child is killed */
            *stat_loc = (*child).exit;

            arch::syscall_return(curthread!(), (*child).pid as usize);
            proc_reap(child);
            return;
        }

        /*
        if (options & WNOHANG) {
            arch::syscall_return(curthread, 0);
            return;
        }
        */

        while (*child).running != 0 {
            if thread_queue_sleep(&mut (*curproc!()).wait_queue) != 0 {
                arch::syscall_return(curthread!(), -EINTR as usize);
                return;
            }
        }

        *stat_loc = (*child).exit;
        arch::syscall_return(curthread!(), (*child).pid as usize);
        proc_reap(child);
    }
}


unsafe fn sys_write(fd: isize, buf: *mut u8, nbytes: size_t) {
    //syscall_log(LOG_DEBUG, "write(fd=%d, buf=%p, nbytes=%d)\n", fd, buf, nbytes);
    
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


unsafe fn sys_ioctl(fd: isize, request: isize, argp: *mut u8) {
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


unsafe fn sys_sigaction(sig: isize, act: *const SignalAction, oact: *mut SignalAction) {
    //syscall_log(LOG_DEBUG, "sigaction(sig=%d, act=%p, oact=%p)\n",
    //        sig, act, oact);

    if sig < 1 || (sig as usize) > SIG_MAX {
        arch::syscall_return(curthread!(), -EINVAL as usize);
        return;
    }

    if !oact.is_null() {
        //memcpy(oact, &curproc->sigaction[sig], sizeof(struct sigaction));
        *oact = (*curproc!()).sigaction[sig as usize];
    }

    if !act.is_null() {
        //memcpy(&curproc->sigaction[sig], act, sizeof(struct sigaction));
        (*curproc!()).sigaction[sig as usize] = *act;
    }

    arch::syscall_return(curthread!(), 0);
}


unsafe fn sys_readdir(fd: isize, dirent: *mut DirectoryEntry) {
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


unsafe fn sys_mount(args: *mut MountArgs) {
    if (*curproc!()).uid != 0 {
        arch::syscall_return(curthread!(), -EACCES as usize);
        return;
    }

    let fs_type = (*args).fs_type;
    let dir     = (*args).dir;
    let flags   = (*args).flags;
    let data    = (*args).data;

    //syscall_log(LOG_DEBUG, "mount(type=%s, dir=%s, flags=%x, data=%p)\n",
    //        type, dir, flags, data);

    let err = vfs_mount(fs_type, dir, flags, data, &mut proc_uio!(curproc!()));
    arch::syscall_return(curthread!(), err as usize);
    return;
}


unsafe fn sys_mkdir(path: *const u8, mode: mode_t) {
    //syscall_log(LOG_DEBUG, "mkdir(path=%s, mode=%x)\n", path, mode);

    let mut uio = proc_uio!(curproc!());

    let mut err = vfs_mkdir(path, mode, &mut uio, core::ptr::null_mut());
    arch::syscall_return(curthread!(), err as usize);
    return;
}


/* XXX */
const UTSNAME_SYSNAME:  *const u8 = "AquilaOS\0".as_ptr();
const UTSNAME_RELEASE:  *const u8 = "v0.1.0\0".as_ptr();
const UTSNAME_NODENAME: *const u8 = "aquila\0".as_ptr();
const UTSNAME_VERSION:  *const u8 = "(rustc 1.45.0-nightly (7ebd87a7a 2020-05-08))\0".as_ptr();
const UTSNAME_MACHINE:  *const u8 = "i386\0".as_ptr();


unsafe fn sys_uname(name: *mut UtsName) {
    //syscall_log(LOG_DEBUG, "uname(name=%p)\n", name);

    /* FIXME: Sanity checking */

    strcpy((*name).sysname.as_mut_ptr(),  UTSNAME_SYSNAME);
    strcpy((*name).nodename.as_mut_ptr(), UTSNAME_NODENAME);
    strcpy((*name).release.as_mut_ptr(),  UTSNAME_RELEASE);
    strcpy((*name).version.as_mut_ptr(),  UTSNAME_VERSION);
    strcpy((*name).machine.as_mut_ptr(),  UTSNAME_MACHINE);

    arch::syscall_return(curthread!(), 0);
    return;
}


unsafe fn sys_pipe(fd: &mut [isize; 2]) {
    //syscall_log(LOG_DEBUG, "pipe(fd=%p)\n", fd);

    let fd1 = proc_fd_get(curproc!());
    let fd2 = proc_fd_get(curproc!());

    pipefs_pipe((*curproc!()).fds.offset(fd1), (*curproc!()).fds.offset(fd2));

    fd[0] = fd1;
    fd[1] = fd2;

    arch::syscall_return(curthread!(), 0);
}


unsafe fn sys_fcntl(fd: isize, cmd: isize, arg: usize) {
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


unsafe fn sys_chdir(path: *const u8) {
    //syscall_log(LOG_DEBUG, "chdir(path=%s)\n", path);

    if path.is_null() || *path == 0 {
        arch::syscall_return(curthread!(), -ENOENT as usize);
        return;
    }

    let mut err = 0;
    let mut abs_path = core::ptr::null_mut();

    let mut vnode = core::ptr::null_mut();
    err = vfs_lookup(path, &mut proc_uio!(curproc!()), &mut vnode, &mut abs_path);

    if err != 0 {
        kfree(abs_path);
        arch::syscall_return(curthread!(), err as usize);
    }

    if !S_ISDIR!((*vnode).mode) {
        err = -ENOTDIR;
        kfree(abs_path);
        arch::syscall_return(curthread!(), err as usize);
    }

    kfree((*curproc!()).cwd);
    (*curproc!()).cwd = strdup(abs_path);

    kfree(abs_path);
    arch::syscall_return(curthread!(), err as usize);
}


unsafe fn sys_getcwd(buf: *mut u8, size: size_t) {
    //syscall_log(LOG_DEBUG, "getcwd(buf=%p, size=%d)\n", buf, size);

    if size == 0 {
        arch::syscall_return(curthread!(), -EINVAL as usize);
        return;
    }

    let len = strlen((*curproc!()).cwd);

    if size < len + 1 {
        arch::syscall_return(curthread!(), -ERANGE as usize);
        return;
    }

    memcpy(buf, (*curproc!()).cwd, len + 1);
    arch::syscall_return(curthread!(), 0);
}

#[repr(C)]
pub struct ThreadArgs {
    stack: usize,

    /* sys entry */
    entry: usize,

    /* user entry */
    uentry: usize,

    arg: usize,
    attr: usize,
}


unsafe fn sys_thread_create(args: *const ThreadArgs) {
    //syscall_log(LOG_DEBUG,
    //        "thread_create(stack=%p, entry=%p, uentry=%p, arg=%p, attr=%p)\n",
    //        __uthread->stack, __uthread->entry, __uthread->uentry,
    //        __uthread->arg, __uthread->attr);

    let mut thread = core::ptr::null_mut();
    thread_create(curthread!(), (*args).stack, (*args).entry, (*args).uentry, (*args).arg, (*args).attr, &mut thread);
    sched_thread_ready(thread);
    arch::syscall_return(curthread!(), (*thread).tid as usize);
}


unsafe fn sys_thread_exit(value_ptr: *mut u8) {
    //syscall_log(LOG_DEBUG, "thread_exit(value_ptr=%p)\n", value_ptr);

    //curthread->value_ptr = value_ptr;
    let owner = curproc!();

    (*curthread!()).kill();

    /* wakeup owner if it is waiting for joining */
    thread_queue_wakeup(&mut (*owner).thread_join);

    arch_sleep();

    loop {}
}


unsafe fn sys_thread_join(tid: isize, value_ptr: *mut *mut u8) {
    //syscall_log(LOG_DEBUG, "thread_join(tid=%d, value_ptr=%p)\n", tid, value_ptr);

    let owner = curproc!();
    let mut thread = core::ptr::null_mut();

    for node in (*owner).threads.iter() {
        let _thread = (*node).value as *mut Thread;

        if (*_thread).tid == tid {
            thread = _thread;
        }
    }

    /* no such thread */
    if thread.is_null() {
        arch::syscall_return(curthread!(), -ECHILD as usize);
        return;
    }

    if (*thread).state == ThreadState::ZOMBIE {
        /* thread is terminated */
        //*value_ptr = thread->value_ptr;
        arch::syscall_return(curthread!(), (*thread).tid as usize);
        return;
    }

    while (*thread).state != ThreadState::ZOMBIE {
        if thread_queue_sleep(&mut (*curproc!()).thread_join) != 0 {
            arch::syscall_return(curthread!(), -EINTR as usize);
            return;
        }
    }

    //*value_ptr = thread->value_ptr;
    arch::syscall_return(curthread!(), (*thread).tid as usize);
}


unsafe fn sys_setpgid(pid: pid_t, pgid: pid_t) {
    //syscall_log(LOG_DEBUG, "setpgid(pid=%d, pgid=%d)\n", pid, pgid);

    //if (pid == 0 && pgid == 0) {
        let err = pgrp_new(curproc!(), core::ptr::null_mut());
        arch::syscall_return(curthread!(), err as usize);
    //} else {
    //    panic("Unsupported");
    //}
}


unsafe fn sys_mknod(path: *const u8, mode: mode_t, dev: dev_t) {
    //syscall_log(LOG_DEBUG, "mknod(path=%s, mode=%x, dev=%x)\n", path, mode, dev);

    let err = vfs_mknod(path, mode, dev, &mut proc_uio!(curproc!()), core::ptr::null_mut());
    arch::syscall_return(curthread!(), err as usize);
}


unsafe fn sys_lstat(path: *const u8, buf: *mut Stat) {
    //syscall_log(LOG_DEBUG, "lstat(path=%s, buf=%p)\n", path, buf);

    let mut vnode = core::ptr::null_mut();
    let mut uio = proc_uio!(curproc!());
    uio.flags = O_NOFOLLOW;

    let mut err = vfs_lookup(path, &mut uio, &mut vnode, core::ptr::null_mut());
    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    err = vfs_stat(vnode, buf);
    arch::syscall_return(curthread!(), err as usize);
}


unsafe fn sys_auth(uid: uid_t, pw: *const u8) {
    //syscall_log(LOG_DEBUG, "auth(uid=%d, pw=%s)\n", uid, pw);

    (*curproc!()).uid = uid;   /* XXX */
    arch::syscall_return(curthread!(), 0);
}


unsafe fn sys_getuid() {
    //syscall_log(LOG_DEBUG, "getuid()\n");
    arch::syscall_return(curthread!(), (*curproc!()).uid as usize);
}


unsafe fn sys_getgid() {
    //syscall_log(LOG_DEBUG, "getgid()\n");
    arch::syscall_return(curthread!(), (*curproc!()).gid as usize);
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


unsafe fn sys_mmap(args: *mut MmapArgs, ret: *mut *mut u8) {
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

    let vm_entry = vm_entry_new();

    if vm_entry.is_null() {
        err = -ENOMEM;
        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    /* initialize vm entry */
    (*vm_entry).base   = (*args).addr;
    (*vm_entry).size   = (*args).len;
    (*vm_entry).flags  = if (*args).prot & PROT_READ  != 0 { VM_UR } else { 0 };
    (*vm_entry).flags |= if (*args).prot & PROT_WRITE != 0 { VM_UW } else { 0 };
    (*vm_entry).flags |= if (*args).prot & PROT_EXEC  != 0 { VM_UX } else { 0 };
    (*vm_entry).flags |= if (*args).flags & MAP_SHARED != 0 { VM_SHARED } else { 0 };
    (*vm_entry).off    = (*args).off as usize;

    (*vm_entry).vm_object = vm_object_vnode((*file).backend.vnode);

    if (*args).flags & MAP_FIXED == 0 {
        /* allocate memory region */
        (*vm_entry).base = 0;
    }

    let vm_space = &mut (*curproc!()).vm_space;

    err = vm_space.insert(&mut *vm_entry);
    if err != 0 {
        if !(*vm_entry).qnode.is_null() {
            (*vm_space).vm_entries.node_remove((*vm_entry).qnode);
        }

        kfree(vm_entry as *mut u8);

        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    if (*args).flags & MAP_PRIVATE == 0 {
        err = vfs_map(vm_space, vm_entry);

        if err != 0 {
            if !(*vm_entry).qnode.is_null() {
                (*vm_space).vm_entries.node_remove((*vm_entry).qnode);
            }

            kfree(vm_entry as *mut u8);

            arch::syscall_return(curthread!(), err as usize);
            return;
        }
    }

    *ret = (*vm_entry).base as *mut u8;

    arch::syscall_return(curthread!(), err as usize);
    return;
}


unsafe fn sys_munmap(addr: usize, len: size_t) {
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


unsafe fn sys_socket(domain: isize, sock_type: isize, protocol: isize) {
    //syscall_log(LOG_DEBUG, "socket(domain=%d, type=%d, protocol=%d)\n",
    //        domain, type, protocol);

    /* find a free file descriptor */
    let fd = proc_fd_get(curproc!());

    if fd == -1 {
        /* reached maximum number of open file descriptors */
        arch::syscall_return(curthread!(), -EMFILE as usize);
        return;
    }
    
    let mut err = 0;
    let file = (*curproc!()).fds.offset(fd);

    err = socket_create(file, domain, sock_type, protocol);
    if err != 0 {
        proc_fd_release(curproc!(), fd);
        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    arch::syscall_return(curthread!(), fd as usize);
    return;
}


unsafe fn sys_accept(fd: isize, addr: *const SocketAddress, len: *mut socklen_t) {
    //syscall_log(LOG_DEBUG, "accept(fd=%d, addr=%p, len=%p)\n",
    //        fd, addr, len);

    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let conn_fd = proc_fd_get(curproc!());

    if conn_fd == -1 {
        arch::syscall_return(curthread!(), -EMFILE as usize);
        return; 
    }

    let socket = (*curproc!()).fds.offset(fd);
    let conn   = (*curproc!()).fds.offset(conn_fd);

    let mut err = 0;
    err = socket_accept(socket, conn, addr, len as usize as u32);

    if err != 0 {
        proc_fd_release(curproc!(), conn_fd);
        arch::syscall_return(curthread!(), err as usize);
        return; 
    }

    arch::syscall_return(curthread!(), conn_fd as usize);
    return;
}


unsafe fn sys_bind(fd: isize, addr: *const SocketAddress, len: socklen_t) {
    //syscall_log(LOG_DEBUG, "bind(fd=%d, addr=%p, len=%d)\n",
    //        fd, addr, len);

    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fd);

    if file.is_null() {
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let mut err = 0;

    err = socket_bind(file, addr, len as usize);
    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return; 
    }

    arch::syscall_return(curthread!(), 0);
    return;
}


unsafe fn sys_connect(fd: isize, addr: *const SocketAddress, len: socklen_t) {
    //syscall_log(LOG_DEBUG, "connect(fd=%d, addr=%p, len=%d)\n",
    //        fd, addr, len);

    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let socket = (*curproc!()).fds.offset(fd);
    let mut err = 0;

    err = socket_connect(socket, addr, len as usize);
    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return; 
    }

    arch::syscall_return(curthread!(), 0);
    return;
}


unsafe fn sys_listen(fd: isize, backlog: isize) {
    //syscall_log(LOG_DEBUG, "listen(fd=%d, backlog=%d)\n", fd, backlog);

    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fd);

    if file.is_null() {
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let mut err = 0;

    err = socket_listen(file, backlog);
    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return; 
    }

    arch::syscall_return(curthread!(), 0);
    return;
}

#[repr(C)]
pub struct SocketIO {
    fd: isize,
    buf: *mut u8,
    len: size_t,
    flags: isize,
}


unsafe fn sys_send(s: *const SocketIO) {
    let fd    = (*s).fd;
    let buf   = (*s).buf;
    let len   = (*s).len;
    let flags = (*s).flags;

    //syscall_log(LOG_DEBUG, "send(fd=%d, buf=%p, len=%d, flags=%x)\n",
    //        fd, buf, len, flags);

    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fd);

    if file.is_null() {
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let mut err = 0;

    err = socket_send(file, buf, len, flags);
    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return; 
    }

    arch::syscall_return(curthread!(), 0);
    return;
}


unsafe fn sys_recv(s: *const SocketIO) {
    let fd    = (*s).fd;
    let buf   = (*s).buf;
    let len   = (*s).len;
    let flags = (*s).flags;

    //syscall_log(LOG_DEBUG, "recv(fd=%d, buf=%p, len=%d, flags=%x)\n",
    //        fd, buf, len, flags);

    if fd < 0 || (fd as usize) >= FDS_COUNT {
        /* out of bounds */
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let file = (*curproc!()).fds.offset(fd);

    if file.is_null() {
        arch::syscall_return(curthread!(), -EBADFD as usize);
        return; 
    }

    let mut err = 0;

    err = socket_recv(file, buf, len, flags);
    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return; 
    }

    arch::syscall_return(curthread!(), 0);
    return;
}


unsafe fn sys_umask(mask: mode_t) {
    //syscall_log(LOG_DEBUG, "umask(mask=%d)\n", mask);

    let cur_mask = (*curproc!()).mask;
    (*curproc!()).mask = mask & 0o0777;

    arch::syscall_return(curthread!(), cur_mask as usize);
    return;
}


unsafe fn sys_chmod(path: *const u8, mode: mode_t) {
    //syscall_log(LOG_DEBUG, "chmod(path=%s, mode=%d)\n", path, mode);

    let mut vnode = core::ptr::null_mut();
    let mut err = 0;
    let mut uio = proc_uio!(curproc!());

    err = vfs_lookup(path, &mut uio, &mut vnode, core::ptr::null_mut());
    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    err = vfs_chmod(vnode, mode);
    arch::syscall_return(curthread!(), err as usize);

    //arch::syscall_return(curthread, -ENOSYS);
}


unsafe fn sys_sysconf(name: isize) {
    //syscall_log(LOG_DEBUG, "sysconf(name=%d)\n", name);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}

const F_OK: usize = 0;
const X_OK: usize = 1;
const W_OK: usize = 2;
const R_OK: usize = 4;


unsafe fn sys_access(path: *const u8, mode: usize) {
    //syscall_log(LOG_DEBUG, "access(path=%s, mode=%d)\n", path, mode);

    let mut err = 0;

    /* look up the file */
    let mut vnode = core::ptr::null_mut();
    let mut uio = proc_uio!(curproc!());

    uio.flags |= if mode & R_OK != 0 { O_RDONLY } else { 0 };
    uio.flags |= if mode & W_OK != 0 { O_WRONLY } else { 0 };
    uio.flags |= if mode & X_OK != 0 { O_EXEC   } else { 0 };

    err = vfs_lookup(path, &mut uio, &mut vnode, core::ptr::null_mut());

    if err != 0 {
        if !vnode.is_null() {
            vfs_close(vnode);
        }

        arch::syscall_return(curthread!(), err as usize);
    }

    let mut file = FileDescriptor {
        backend: FileBackend { vnode },
        offset: 0,
        flags: uio.flags,
    };

    err = vfs_perms_check(&mut file, &mut uio);
    if err != 0 {
        if !vnode.is_null() {
            vfs_close(vnode);
        }

        arch::syscall_return(curthread!(), err as usize);
    }

    arch::syscall_return(curthread!(), err as usize);
    return;
}


unsafe fn sys_gettimeofday(tv: *mut TimeVal, tz: *mut TimeZone) {
    //syscall_log(LOG_DEBUG, "gettimeofday(tv=%p, tz=%p)\n", tv, tz);
    arch::syscall_return(curthread!(), gettimeofday(tv, tz) as usize);
    return;
}


unsafe fn sys_sigmask(how: isize, set: *mut u8, oldset: *mut u8) {
    //syscall_log(LOG_DEBUG, "sigmask(how=%d, set=%p, oldset=%p)\n", how, set, oldset);
    arch::syscall_return(curthread!(), -ENOTSUP as usize);
}

type fd_mask = usize;

const FD_SETSIZE: usize = 64;
const NFDBITS: usize = core::mem::size_of::<fd_mask>() * 8;

//macro NFDBITS {
//    /* bits per mask */
//    () => (core::mem::size_of::<fd_mask>() * 8)
//}

macro _howmany {
    ($x:expr, $y:expr) => (($x + $y - 1) / $y)
}

#[repr(C)]
pub struct FdSet {
    fds_bits: [fd_mask; _howmany!(FD_SETSIZE, NFDBITS)],
}

#[repr(C)]
pub struct SelectArgs {
    nfds: isize,
    readfds: *mut FdSet,
    writefds: *mut FdSet,
    exceptfds: *mut FdSet,
    timeout: *mut TimeVal,
}


unsafe fn sys_select(args: *mut SelectArgs) {
    //syscall_log(LOG_DEBUG, "select(args=%p)\n", args);

    let nfds = (*args).nfds as usize;
    let readfds = (*args).readfds;
    let writefds = (*args).writefds;
    let exceptfds = (*args).exceptfds;
    let timeout = (*args).timeout;

    let mut count = 0;

    for i in 0..nfds {
        if !readfds.is_null() && ((*readfds).fds_bits[i/NFDBITS] & (1 << (i % NFDBITS)) != 0) {
            let file = (*curproc!()).fds.offset(i as isize);
            if vfs_file_can_read(file, 1) > 0 {
                (*readfds).fds_bits[i/NFDBITS] |= (1 << (i % NFDBITS));
                count += 1;
            } else {
                (*readfds).fds_bits[i/NFDBITS] &= !(1 << (i % NFDBITS));
            }
        }

        if !writefds.is_null() && ((*writefds).fds_bits[i/NFDBITS] & (1 << (i % NFDBITS)) != 0) {
            let file = (*curproc!()).fds.offset(i as isize);
            if vfs_file_can_write(file, 1) > 0 {
                (*writefds).fds_bits[i/NFDBITS] |= (1 << (i % NFDBITS));
                count += 1;
            } else {
                (*writefds).fds_bits[i/NFDBITS] &= !(1 << (i % NFDBITS));
            }
        }
    }

    arch::syscall_return(curthread!(), count);
}


unsafe fn sys_getpgrp() {
    //syscall_log(LOG_DEBUG, "getpgrp()\n");
    arch::syscall_return(curthread!(), (*(*curproc!()).pgrp).pgid as usize);
}


unsafe fn sys_chown(path: *const u8, uid: uid_t, gid: gid_t) {
    //syscall_log(LOG_DEBUG, "chown(path=%s, uid=%d, gid=%d)\n", path, uid, gid);

    let mut vnode = core::ptr::null_mut();
    let mut err = 0;
    let mut uio = proc_uio!(curproc!());

    err = vfs_lookup(path, &mut uio, &mut vnode, core::ptr::null_mut());
    if err != 0 {
        arch::syscall_return(curthread!(), err as usize);
        return;
    }

    err = vfs_chown(vnode, uid, gid);
    arch::syscall_return(curthread!(), err as usize);
}


unsafe fn sys_fchown(fd: isize, owner: uid_t, group: gid_t) {
    //syscall_log(LOG_DEBUG, "fchown(fd=%d, owner=%d, group=%d)\n", fd, owner, group);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}


unsafe fn sys_lchown(path: *const u8, owner: uid_t, group: gid_t) {
    //syscall_log(LOG_DEBUG, "lchown(path=%s, owner=%d, group=%d)\n", path, owner, group);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}


unsafe fn sys_utime(path: *const u8, times: *const utimbuf) {
    //syscall_log(LOG_DEBUG, "utime(path=%s, times=%p)\n", path, times);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}


unsafe fn sys_rmdir(path: *const u8) {
    //syscall_log(LOG_DEBUG, "rmdir(path=%s)\n", path);
    arch::syscall_return(curthread!(), -ENOSYS as usize);
}

#[repr(transparent)]
pub struct Syscall(pub *const u8);
unsafe impl Sync for Syscall {}

// XXX find a way to dynamically count syscalls

pub static SYSCALL_TABLE: [Syscall; 60] = [
    /* 00 */    Syscall(core::ptr::null()),
    /* 01 */    Syscall(sys_exit as *const _),
    /* 02 */    Syscall(sys_close as *const _),
    /* 03 */    Syscall(sys_execve as *const _),
    /* 04 */    Syscall(sys_fork as *const _),
    /* 05 */    Syscall(sys_fstat as *const _),
    /* 06 */    Syscall(sys_getpid as *const _),
    /* 07 */    Syscall(sys_isatty as *const _),
    /* 08 */    Syscall(sys_kill as *const _),
    /* 09 */    Syscall(sys_link as *const _),
    /* 10 */    Syscall(sys_lseek as *const _),
    /* 11 */    Syscall(sys_open as *const _),
    /* 12 */    Syscall(sys_read as *const _),
    /* 13 */    Syscall(sys_sbrk as *const _),
    /* 14 */    Syscall(sys_stat as *const _),
    /* 15 */    Syscall(sys_times as *const _),
    /* 16 */    Syscall(sys_unlink as *const _),
    /* 17 */    Syscall(sys_waitpid as *const _),
    /* 18 */    Syscall(sys_write as *const _),
    /* 19 */    Syscall(sys_ioctl as *const _),
    /* 20 */    Syscall(sys_sigaction as *const _),
    /* 21 */    Syscall(sys_readdir as *const _),
    /* 22 */    Syscall(sys_mount as *const _),
    /* 23 */    Syscall(sys_mkdir as *const _),
    /* 24 */    Syscall(sys_uname as *const _),
    /* 25 */    Syscall(sys_pipe as *const _),
    /* 26 */    Syscall(sys_fcntl as *const _),
    /* 27 */    Syscall(sys_chdir as *const _),
    /* 28 */    Syscall(sys_getcwd as *const _),
    /* 29 */    Syscall(sys_thread_create as *const _),
    /* 30 */    Syscall(sys_thread_exit as *const _),
    /* 31 */    Syscall(sys_thread_join as *const _),
    /* 32 */    Syscall(sys_setpgid as *const _),
    /* 33 */    Syscall(sys_mknod as *const _),
    /* 34 */    Syscall(sys_lstat as *const _),
    /* 35 */    Syscall(sys_auth as *const _), /* deprecated */
    /* 36 */    Syscall(sys_getuid as *const _),
    /* 37 */    Syscall(sys_getgid as *const _),
    /* 38 */    Syscall(sys_mmap as *const _),
    /* 39 */    Syscall(sys_munmap as *const _),
    /* 40 */    Syscall(sys_socket as *const _),
    /* 41 */    Syscall(sys_accept as *const _),
    /* 42 */    Syscall(sys_bind as *const _),
    /* 43 */    Syscall(sys_connect as *const _),
    /* 44 */    Syscall(sys_listen as *const _),
    /* 45 */    Syscall(sys_send as *const _),
    /* 46 */    Syscall(sys_recv as *const _),
    /* 47 */    Syscall(sys_umask as *const _),
    /* 48 */    Syscall(sys_chmod as *const _),
    /* 49 */    Syscall(sys_sysconf as *const _),
    /* 50 */    Syscall(sys_gettimeofday as *const _),
    /* 51 */    Syscall(sys_access as *const _),
    /* 52 */    Syscall(sys_sigmask as *const _),
    /* 53 */    Syscall(sys_select as *const _),
    /* 54 */    Syscall(sys_getpgrp as *const _),
    /* 55 */    Syscall(sys_chown as *const _),
    /* 56 */    Syscall(sys_fchown as *const _),
    /* 57 */    Syscall(sys_lchown as *const _),
    /* 58 */    Syscall(sys_utime as *const _),
    /* 59 */    Syscall(sys_rmdir as *const _),
];

//pub static syscall_cnt: size_t = core::mem::size_of_val(&syscall_table)/core::mem::size_of_val(&syscall_table[0]);

pub static SYSCALL_CNT: size_t = 60;
