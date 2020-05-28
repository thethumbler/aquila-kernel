use prelude::*;

use sys::session::*;
use sys::pgroup::*;
use sys::process::*;

use arch::i386::sys::*;

use kern::string::*;
use crate::arch;
use crate::include::core::arch::*;
use crate::include::core::types::*;
use crate::include::bits::errno::*;

use crate::include::fs::vfs::*;
use crate::fs::{Vnode};

use crate::sys::thread::*;

use crate::mm::*;
use crate::include::mm::mm::*;
use crate::include::mm::vm::*;
use crate::include::mm::kvmem::*;

use crate::include::net::socket::*;

use crate::{malloc_declare};

malloc_declare!(M_FDS);

// FIXME
const FDS_COUNT: usize = 64;

pub unsafe fn copy_fds(parent: *mut Process, fork: *mut Process) -> isize {
    /* copy open files descriptors */
    (*fork).fds = kmalloc(FDS_COUNT * core::mem::size_of::<FileDescriptor>(), &M_FDS, 0) as *mut FileDescriptor;

    if (*fork).fds.is_null() {
        return -ENOMEM;
    }

    memcpy((*fork).fds as *mut u8, (*parent).fds as *const u8, FDS_COUNT * core::mem::size_of::<FileDescriptor>());
    //*(*fork).fds = *(*parent).fds;

    for i in 0..FDS_COUNT {
        let file = (*fork).fds.offset(i as isize);
        if !(*file).backend.vnode.is_null() && (*file).backend.vnode != (-1isize) as *mut Vnode {
            if (*file).flags & FILE_SOCKET != 0 {
                (*(*file).backend.socket).refcnt += 1;
            } else {
                (*(*file).backend.vnode).refcnt += 1;
            }
        }
    }

    return 0;
}

pub unsafe fn fork_proc_copy(parent: *mut Process, fork: *mut Process) -> isize {
    (*fork).pgrp = (*parent).pgrp;
    (*fork).pgrp_node = (*(*fork).pgrp).procs.as_mut().unwrap().enqueue(fork);

    (*fork).mask = (*parent).mask;
    (*fork).uid  = (*parent).uid;
    (*fork).gid  = (*parent).gid;

    (*fork).heap_start = (*parent).heap_start;
    (*fork).heap  = (*parent).heap;
    (*fork).entry = (*parent).entry;

    memcpy(&(*fork).sigaction as *const _ as *mut u8, &(*parent).sigaction as *const _ as *const u8, core::mem::size_of_val(&(*parent).sigaction));

    return 0;
}

pub unsafe fn proc_fork(thread: *mut Thread, proc_ref: *mut *mut Process) -> isize {
    let mut err = 0;
    let mut fork: *mut Process = core::ptr::null_mut();
    let mut fork_thread: *mut Thread = core::ptr::null_mut();

    /* create new process for fork */
    err = proc_new(&mut fork);
    if err != 0 {
        //goto error;
        return err;
    }

    /* new process main thread */
    fork_thread = (*(*fork).threads.head).value as *mut Thread;

    /* parent process */
    let mut proc = (*thread).owner;

    /* copy process structure */
    err = fork_proc_copy(proc, fork);
    if err != 0 {
        //goto error;
        //FIXME
        return err;
    }

    /* copy process name */
    (*fork).name = strdup((*proc).name);
    if (*fork).name.is_null() {
        err = -ENOMEM;
        //goto error;
        return err;
    }

    /* Allocate a new PID */
    (*fork).pid = proc_pid_alloc();

    /* Set fork parent */
    (*fork).parent = proc;

    /* mark the new thread as spawned
     * fork continues execution from a spawned thread */
    (*fork_thread).spawned = 1;

    /* copy current working directory */
    (*fork).cwd = strdup((*proc).cwd);
    if (*fork).cwd.is_null() {
        err = -ENOMEM;
        //goto error;
        //FIXME
        return err;
    }

    /* allocate new signals queue */
    (*fork).sig_queue = Some(Queue::alloc());
    if (*fork).sig_queue.is_none() {
        err = -ENOMEM;
        //goto error;
        //FIXME
        return err;
    }

    /* copy file descriptors */
    err = copy_fds(proc, fork);
    if err != 0 {
        //goto error;
        //FIXME
        return err;
    }

    /* copy virtual memory space */
    err = vm_space_fork(&mut (*proc).vm_space, &mut (*fork).vm_space);
    if err != 0 {
        //goto error;
        return err;
    }

    /* fix heap & stack entry pointers -- XXX yes, we are doing this */
    let mut pvm_node = (*proc).vm_space.vm_entries.head;
    let mut fvm_node = (*fork).vm_space.vm_entries.head;

    while !pvm_node.is_null() {
        let pvm_entry = (*pvm_node).value as *mut VmEntry;
        let fvm_entry = (*fvm_node).value as *mut VmEntry;

        if pvm_entry == (*proc).heap_vm {
            (*fork).heap_vm = fvm_entry;
        }

        if pvm_entry == (*proc).stack_vm {
            (*fork).stack_vm = fvm_entry;
        }

        pvm_node = (*pvm_node).next;
        fvm_node = (*fvm_node).next;
    }

    /* call arch specific fork handler */
    err = arch::proc_fork(thread, fork);

    if err == 0 {
        /* return 0 to child */
        arch_syscall_return(fork_thread, 0);
        /* and pid to parent */
        arch_syscall_return(thread, (*fork).pid as usize);
    } else {
        /* return error to parent */
        arch_syscall_return(thread, err as usize);
        //goto error;
        return err;
    }

    if !proc_ref.is_null() {
        *proc_ref = fork;
    }

    return 0;

//error:
//    if (fork) {
//        if (fork->name)
//            kfree(fork->name);
//        if (fork->cwd)
//            kfree(fork->cwd);
//        if (fork->sig_queue)
//            kfree(fork->sig_queue);
//
//        /* TODO free VMRs */
//
//        kfree(fork);
//    }
//
//    return err;
}

