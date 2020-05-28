use prelude::*;

use sys::process::*;
use sys::pgroup::*;

malloc_define!(M_SESSION, "session\0", "session structure\0");

/* all sessions */
static mut sessions_queue: Queue<*mut Session> = Queue::empty();
pub static mut sessions: *mut Queue<*mut Session> = unsafe { &mut sessions_queue };

pub struct Session {
    /** session id */
    pub sid: pid_t,

    /** process groups */
    pub pgps: Option<Box<Queue<*mut ProcessGroup>>>,

    /** session leader */
    pub leader: *mut Process,

    /* controlling terminal */
    pub ctty: *mut u8,

    /** session node on sessions queue */
    pub qnode: *mut QueueNode<*mut Session>,
}

unsafe impl Sync for Session {}

pub unsafe fn session_new(proc: *mut Process) -> isize {
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

    (*session).pgps = Some(Queue::alloc());
    if (*session).pgps.is_none() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*pgrp).procs = Some(Queue::alloc());
    if (*pgrp).procs.is_none() {
        //goto e_nomem;
        //FIXME
        return -ENOMEM;
    }

    (*pgrp).session_node = (*session).pgps.as_mut().unwrap().enqueue(pgrp);
    if (*pgrp).session_node.is_null() {
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

    (*session).sid = (*proc).pid;
    (*pgrp).pgid = (*proc).pid;

    (*session).leader = proc;
    (*pgrp).leader = proc;

    (*pgrp).session = session;
    (*proc).pgrp = pgrp;

    return 0;
}

