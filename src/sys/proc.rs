use prelude::*;

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
malloc_define!(M_SESSION, "session\0", "session structure\0");
malloc_define!(M_PGROUP, "pgroup\0", "process group structure\0");
malloc_define!(M_FDS, "fds\0", "file descriptor array\0"); /* FIXME */


/**
 * \ingroup sys
 * \brief session
 */
#[repr(C)]
pub struct Session {
    /** session id */
    pub sid: pid_t,

    /** process groups */
    pub pgps: *mut Queue<*mut ProcessGroup>,

    /** session leader */
    pub leader: *mut Process,

    /* controlling terminal */
    pub ctty: *mut u8,

    /** session node on sessions queue */
    pub qnode: *mut QueueNode<*mut Session>,
}

unsafe impl Sync for Session {}

/**
 * \ingroup sys
 * \brief process Group
 */
#[repr(C)]
pub struct ProcessGroup {
    /** process group id */
    pub pgid: pid_t,

    /** associated session */
    pub session: *mut Session,

    /** session queue node */
    pub session_node: *mut QueueNode<*mut ProcessGroup>,

    /** processes */
    pub procs: *mut Queue<*mut Process>,

    /** process group leader */
    pub leader: *mut Process,

    /** group node on pgroups queue */
    pub qnode: *mut QueueNode<*mut ProcessGroup>,
}

unsafe impl Sync for ProcessGroup {}

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
    pub sig_queue: *mut Queue<isize>,

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

/* all processes */
static mut procs_queue: Queue<*mut Process> = Queue::empty();
pub static mut procs: *mut Queue<*mut Process> = unsafe { &mut procs_queue };

/* all sessions */
static mut sessions_queue: Queue<*mut Session> = Queue::empty();
pub static mut sessions: *mut Queue<*mut Session> = unsafe { &mut sessions_queue };

/* all process groups */
static mut pgroups_queue: Queue<*mut ProcessGroup> = Queue::empty();
pub static mut pgroups: *mut Queue<*mut ProcessGroup> = unsafe { &mut pgroups_queue };

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

    err = thread_new(proc, &mut thread);

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
    (*procs).enqueue(proc);

    if !proc_ref.is_null() {
        *proc_ref = proc;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn proc_pid_find(pid: pid_t) -> *mut Process {
    //queue_for (node, procs) {
    for qnode in (*procs).iter() {
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

        if !(*proc).sig_queue.is_null() {
            kfree((*proc).sig_queue as *mut u8);
        }

        return err;
    }

    /* initalize signals queue */
    (*proc).sig_queue = Queue::new();

    if (*proc).sig_queue.is_null() {
        err = -ENOMEM;

        kfree((*proc).fds as *mut u8);
        kfree((*proc).sig_queue as *mut u8);

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

    /* Kill all threads */
    while (*proc).threads.count > 0 {
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

        thread_kill(thread);
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

    while (*(*proc).sig_queue).count > 0 {
        (*(*proc).sig_queue).dequeue();
    }

    kfree((*proc).sig_queue as *mut u8);

    /* Mark all children as orphans */
    for qnode in (*procs).iter() {
        let _proc = (*qnode).value as *mut Process;

        if (*_proc).parent == proc {
            (*_proc).parent = core::ptr::null_mut();
        }
    }

    kfree((*proc).name as *mut u8);

    /* XXX */
    (*(*(*proc).pgrp).procs).node_remove((*proc).pgrp_node);

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

    (*procs).remove(proc);
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

#[no_mangle]
pub unsafe extern "C" fn session_new(proc: *mut Process) -> isize {
    let mut err = 0;

    let mut session: *mut Session = core::ptr::null_mut();
    let mut pgrp: *mut ProcessGroup = core::ptr::null_mut();

    /* allocate a new session structure */
    session = kmalloc(core::mem::size_of::<Session>(), &M_SESSION, M_ZERO) as *mut Session;
    if session.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    /* allocate a new process group structure for the session */
    pgrp = kmalloc(core::mem::size_of::<ProcessGroup>(), &M_PGROUP, M_ZERO) as *mut ProcessGroup;
    if pgrp.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*session).pgps = Queue::new();
    if (*session).pgps.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*pgrp).procs = Queue::new();
    if (*pgrp).procs.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*pgrp).session_node = (*(*session).pgps).enqueue(pgrp);
    if (*pgrp).session_node.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*proc).pgrp_node = (*(*pgrp).procs).enqueue(proc);
    if (*proc).pgrp_node.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*session).sid = (*proc).pid;
    (*pgrp).pgid = (*proc).pid;

    (*session).leader = proc;
    (*pgrp).leader = proc;

    (*pgrp).session = session;
    (*proc).pgrp = pgrp;

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn pgrp_new(proc: *mut Process, pgroup_ref: *mut *mut ProcessGroup) -> isize {
    let mut err = 0;
    let mut pgrp: *mut ProcessGroup = core::ptr::null_mut();
    
    pgrp = kmalloc(core::mem::size_of::<ProcessGroup>(), &M_PGROUP, M_ZERO) as *mut ProcessGroup;
    if pgrp.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*pgrp).pgid = (*proc).pid;
    (*pgrp).session = (*(*proc).pgrp).session;

    /* remove the process from the old process group */
    (*(*(*proc).pgrp).procs).node_remove((*proc).pgrp_node);

    (*pgrp).procs = Queue::new();
    if (*pgrp).procs.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*proc).pgrp_node = (*(*pgrp).procs).enqueue(proc);
    if (*proc).pgrp_node.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*pgrp).session_node = (*(*(*(*proc).pgrp).session).pgps).enqueue(pgrp);
    if (*pgrp).session_node.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    if (*(*(*proc).pgrp).procs).count == 0 {
        /* TODO */
    }

    (*proc).pgrp = pgrp;

    if !pgroup_ref.is_null() {
        *pgroup_ref = pgrp;
    }

    return 0;
}
