use prelude::*;

use sys::session::*;
use sys::pgroup::*;

use arch::i386::platform::pc::reboot::arch_reboot;
use arch::i386::sys::*;
use arch::i386::mm::i386::*;
use crate::include::bits::errno::*;

use crate::include::core::types::*;
use crate::include::core::string::*;

use crate::sys::signal::*;
use crate::sys::thread::*;

use crate::mm::*;
use crate::include::mm::vm::*;
use crate::include::mm::kvmem::*;
use crate::include::mm::pmap::*;

use crate::include::fs::vfs::*;
use crate::include::core::types::*;
use crate::sys::signal::*;
use crate::sys::thread::*;
use crate::include::fs::vfs::*;

use crate::mm::*;
use crate::include::mm::vm::*;

use crate::fs::{Vnode};

use crate::{curthread, malloc_define, bitmap_new, print};

malloc_define!(M_PROC, "proc\0", "process structure\0");
malloc_define!(M_FDS, "fds\0", "file descriptor array\0"); /* FIXME */

#[derive(Debug)]
pub struct Process {
    /** process id */
    pub pid: pid_t,

    /** associated process group */
    pub pgrp: *mut ProcessGroup,
    pub pgrp_node: *mut QueueNode<*mut Process>,

    /** process name - XXX */
    pub name: *mut u8,

    /** open file descriptors */
    pub fds: *mut FileDescriptor,

    /** parent process */
    pub parent: *mut Process,

    /** current working directory */
    pub cwd: *mut u8, //char *cwd;

    /** file mode creation mask */
    pub mask: mode_t,

    /** user id */
    pub uid: uid_t,

    /** groupd id */
    pub gid: gid_t,

    /** process initial heap pointer */
    pub heap_start: usize,

    /** process current heap pointer */
    pub heap: usize,

    /** process entry point */  
    pub entry: usize,

    /** virtual memory regions */
    pub vm_space: AddressSpace,

    pub heap_vm: *mut VmEntry,
    pub stack_vm: *mut VmEntry,

    /** process threads */
    pub threads: Queue<*mut Thread>,

    /** threads join wait queue */
    pub thread_join: Queue<*mut Thread>,

    /** recieved signals queue */
    pub sig_queue: Option<Box<Queue<isize>>>,

    /** dummy queue for children wait */
    pub wait_queue: Queue<*mut Thread>,

    /** registered signal handlers */
    pub sigaction: [SignalAction; SIG_MAX + 1],

    /** exit status of process */
    pub exit: isize,

    /** process is running? */
    pub running: isize,
}

pub macro proc_exit {
    ($info:expr, $code:expr) => {
        (((($info) & 0xff) << 8) | (($code) & 0xff))
    }
}

pub macro proc_uio {
    ($proc:expr) => {
        UserOp {
            cwd:  (*$proc).cwd,
            uid:  (*$proc).uid,
            gid:  (*$proc).gid,
            mask: (*$proc).mask,
            flags: 0,
            root: core::ptr::null_mut(),
        }
    }
}

impl Process {
    pub fn new_thread(&mut self, thread_ref: *mut *mut Thread) -> isize {
        // XXX
        let mut thread = unsafe { Box::leak(Box::<Thread>::new_zeroed_tagged(&M_THREAD).assume_init()) };

        thread.owner = self;
        thread.tid = (self.threads.count() + 1) as tid_t;

        unsafe { self.threads.enqueue(thread); }

        if !thread_ref.is_null() {
            unsafe { *thread_ref = thread; }
        }

        return 0;
    }
}

/* all processes */
pub static mut procs: Queue<*mut Process> = Queue::empty();

//static mut pid_bitmap: *mut BitMap = &BitMap::empty(4096) as *const _ as *mut BitMap;
static mut pid_bitmap: *mut BitMap = &bitmap_new!(4096) as *const _ as *mut BitMap;
static mut ff_pid: isize = 1;

pub unsafe fn proc_pid_alloc() -> isize {
    for i in (ff_pid as usize)..(*pid_bitmap).max_idx {
        if bitmap_check(pid_bitmap, i) == 0 {
            bitmap_set(pid_bitmap, i);
            ff_pid = i as isize;
            return i as isize;
        }
    }

    return -1;
}

pub unsafe fn proc_pid_free(pid: isize) {
    bitmap_clear(pid_bitmap, pid as usize);

    if pid < ff_pid {
        ff_pid = pid;
    }
}

pub unsafe fn proc_new(proc_ref: *mut *mut Process) -> isize {
    let mut err = 0;

    let mut proc: *mut Process = core::ptr::null_mut();
    let mut thread = core::ptr::null_mut();
    let mut pmap = core::ptr::null_mut();

    let proc = kmalloc(core::mem::size_of::<Process>(), &M_PROC, M_ZERO) as *mut Process;

    if proc.is_null() {
        return -ENOMEM;
    }

    err = (*proc).new_thread(&mut thread);

    if err != 0 {
        kfree(proc as *mut u8);
        return err;
    }

    pmap = pmap_create();

    if pmap.is_null() {
        err = -ENOMEM;

        kfree(thread as *mut u8);
        kfree(proc as *mut u8);

        return err;
    }

    (*proc).vm_space.pmap = pmap;

    /* Set all signal handlers to default */
    for i in 0..SIG_MAX {
        //FIXME
        //(*proc).sigaction[i].sa_handler = SIG_DFL;
    }

    (*proc).running = 1;

    /* add process to all processes queue */
    procs.enqueue(proc);

    if !proc_ref.is_null() {
        *proc_ref = proc;
    }

    return 0;
}


#[no_mangle]
pub unsafe extern "C" fn proc_pid_find(pid: pid_t) -> *mut Process {
    //queue_for (node, procs) {
    for qnode in procs.iter() {
        let proc = (*qnode).value as *mut Process;

        if (*proc).pid == pid {
            return proc;
        }
    }
    
    return core::ptr::null_mut();
}

// FIXME
const FDS_COUNT: usize = 64;

#[no_mangle]
pub unsafe extern "C" fn proc_init(proc: *mut Process) -> isize {
    if proc.is_null() {
        return -EINVAL;
    }

    let mut err = 0;

    (*proc).pid = proc_pid_alloc();
    (*proc).fds = kmalloc(FDS_COUNT * core::mem::size_of::<FileDescriptor>(), &M_FDS, M_ZERO) as *mut FileDescriptor;

    if (*proc).fds.is_null() {
        err = -ENOMEM;

        if !(*proc).sig_queue.is_none() {
            core::mem::take(&mut (*proc).sig_queue);
        }

        return err;
    }

    /* initalize signals queue */
    (*proc).sig_queue = Some(Queue::alloc());

    if (*proc).sig_queue.is_none() {
        err = -ENOMEM;

        kfree((*proc).fds as *mut u8);
        return err;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn proc_kill(proc: *mut Process) {
    if (*proc).pid == 1 {
        if (*proc).exit != 0 {
            panic!("init killed");
        }

        print!("kernel: reached target reboot\n");
        arch_reboot();

        panic!("reboot not implemented\n");
    }

    (*proc).running = 0;

    let mut kill_curthread = 0;

    /* kill all threads */
    while (*proc).threads.count() > 0 {
        let thread = (*proc).threads.dequeue().unwrap();

        if !(*thread).sleep_node.is_null() {
            /* thread is sleeping on some queue */
            (*(*thread).sleep_queue).node_remove((*thread).sleep_node);
        }

        if !(*thread).sched_node.is_null() {
            /* thread is in the scheduler queue */
            (*(*thread).sched_queue).node_remove((*thread).sched_node);
        }

        if thread == curthread!() {
            kill_curthread = 1;
            continue;
        }

        (*thread).kill();
        kfree(thread as *mut u8);
    }

    /* close all file descriptors */
    for i in 0..FDS_COUNT {
        //let file = &(*proc).fds[i];
        //if (*file).backend.vnode && (*file).vnode != -1 as *const _ {
        //    vfs_file_close(file);
        //    file->vnode = NULL;
        //}
    }

    let vm_space = &mut (*proc).vm_space;

    vm_space_destroy(vm_space);
    pmap_decref((*vm_space).pmap);

    /* Free kernel-space resources */
    kfree((*proc).fds as *mut u8);
    kfree((*proc).cwd as *mut u8);

    while (*proc).sig_queue.as_ref().unwrap().count() > 0 {
        (*proc).sig_queue.as_mut().unwrap().dequeue();
    }

    // XXX
    core::mem::take(&mut (*proc).sig_queue);

    /* mark all children as orphans */
    for qnode in procs.iter() {
        let _proc = (*qnode).value as *mut Process;

        if (*_proc).parent == proc {
            (*_proc).parent = core::ptr::null_mut();
        }
    }

    kfree((*proc).name as *mut u8);

    /* XXX */
    (*(*proc).pgrp).procs.as_mut().unwrap().node_remove((*proc).pgrp_node);

    /* Wakeup parent if it is waiting for children */
    if !(*proc).parent.is_null() {
        thread_queue_wakeup(&mut (*(*proc).parent).wait_queue);
        signal_proc_send((*proc).parent, SIGCHLD);
    } else { 
        /* Orphan zombie, just reap it */
        proc_reap(proc);
    }

    if kill_curthread != 0 {
        arch_cur_thread_kill();
        panic!("how did we get here?");
    }
}

#[no_mangle]
pub unsafe extern "C" fn proc_reap(proc: *mut Process) -> isize {
    proc_pid_free((*proc).pid);

    procs.remove(proc);
    kfree(proc as *mut u8);

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn proc_fd_get(proc: *mut Process) -> isize {
    for i in 0..FDS_COUNT {
        if (*(*proc).fds.offset(i as isize)).backend.vnode.is_null() {
            (*(*proc).fds.offset(i as isize)).backend.vnode = (-1isize as usize) as *mut Vnode;
            return i as isize;
        }
    }

    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn proc_fd_release(proc: *mut Process, fd: isize) {
    if (fd as usize) < FDS_COUNT {
        (*(*proc).fds.offset(fd)).backend.vnode = core::ptr::null_mut();
    }
}