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

impl ProcessGroup {
    pub fn alloc() -> Box<ProcessGroup> {
        unsafe { Box::new_zeroed_tagged(&M_PGROUP).assume_init() }
    }
}

pub unsafe fn pgrp_new(proc: *mut Process, pgroup_ref: *mut *mut ProcessGroup) -> isize {
    let mut err = 0;
    let mut pgrp = Box::leak(ProcessGroup::alloc());

    pgrp.pgid = (*proc).pid;
    pgrp.session = (*(*proc).pgrp).session;

    /* remove the process from the old process group */
    (*(*proc).pgrp).procs.as_mut().unwrap().node_remove((*proc).pgrp_node);

    pgrp.procs = Some(Queue::alloc(Queue::new()));

    (*proc).pgrp_node = (*pgrp).procs.as_mut().unwrap().enqueue(proc);
    if (*proc).pgrp_node.is_null() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    pgrp.session_node = (*(*(*proc).pgrp).session).pgps.as_mut().unwrap().enqueue(pgrp);
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
