use prelude::*;

use sys::session::*;
use sys::process::*;

malloc_define!(M_PGROUP, "pgroup\0", "process group structure\0");

/* all process groups */
pub static mut PGROUPS: Queue<*mut ProcessGroup> = Queue::empty();

pub struct ProcessGroup {
    /** process group id */
    pub pgid: pid_t,

    /** associated session */
    pub session: *mut Session,

    /** session queue node */
    pub session_node: *mut QueueNode<*mut ProcessGroup>,

    /** processes */
    pub procs: Option<Box<Queue<*mut Process>>>,

    /** process group leader */
    pub leader: *mut Process,

    /** group node on pgroups queue */
    pub qnode: *mut QueueNode<*mut ProcessGroup>,
}

unsafe impl Sync for ProcessGroup {}

pub unsafe fn pgrp_new(proc: *mut Process, pgroup_ref: *mut *mut ProcessGroup) -> isize {
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
    (*(*proc).pgrp).procs.as_mut().unwrap().node_remove((*proc).pgrp_node);

    (*pgrp).procs = Some(Queue::alloc());
    if (*pgrp).procs.is_none() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*proc).pgrp_node = (*pgrp).procs.as_mut().unwrap().enqueue(proc);
    if (*proc).pgrp_node.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*pgrp).session_node = (*(*(*proc).pgrp).session).pgps.as_mut().unwrap().enqueue(pgrp);
    if (*pgrp).session_node.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    if (*(*proc).pgrp).procs.as_ref().unwrap().count() == 0 {
        /* TODO */
    }

    (*proc).pgrp = pgrp;

    if !pgroup_ref.is_null() {
        *pgroup_ref = pgrp;
    }

    return 0;
}
