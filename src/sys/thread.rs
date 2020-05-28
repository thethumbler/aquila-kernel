use prelude::*;

use sys::process::*;
use sys::sched::*;

use arch::i386::sys::*;
use core::fmt;

use crate::{malloc_define, curthread};

malloc_define!(M_THREAD, "thread\0", "thread structure\0");

#[repr(C)]
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ThreadState {
    RUNNABLE = 1,
    ISLEEP = 2, /* Interruptable SLEEP (I/O) */
    USLEEP = 3, /* Uninterruptable SLEEP (Waiting for event) */
    ZOMBIE = 4,
}

#[repr(C)]
pub struct ThreadStack {
    pub pointer: usize,
    pub base: usize,
    pub size: usize,
}

impl fmt::Debug for ThreadStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::write(f, core::format_args!("ThreadStack {{ pointer: {:p}, base: {:p}, size: {:#x} }}",
                self.pointer as *const u8,
                self.base as *const u8,
                self.size
            )
        )
    }
}

/**
 * \ingroup sys
 * \brief thread
 */
#[repr(C)]
pub struct Thread {
    /** Thread ID */
    pub tid: tid_t,

    /** Thread current state */
    pub state: ThreadState,

    /** Thread owner process */
    pub owner: *mut Process,

    /** Thread stack */
    pub stack: ThreadStack,

    /** Current sleep queue */
    pub sleep_queue: *mut Queue<*mut Thread>,
    pub sleep_node: *mut QueueNode<*mut Thread>,

    /** Scheduler queue */
    pub sched_queue: *mut Queue<*mut Thread>,
    pub sched_node: *mut QueueNode<*mut Thread>,

    /** Arch specific data */
    pub arch: *mut u8,

    /** Thread flags */
    pub spawned: isize,
}

unsafe impl Sync for Thread {}

impl fmt::Debug for Thread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::write(f, core::format_args!("Thread {{ tid: {}, state: {:?}, owner: {:p}, stack: {:?} }}",
                self.tid,
                self.state as i32,
                self.owner,
                self.stack
            )
        )
    }
}

pub unsafe fn thread_new(proc: *mut Process, thread_ref: *mut *mut Thread) -> isize {
    if proc.is_null() {
        //return -EINVAL;
        return -1;
    }

    let mut thread: *mut Thread = core::ptr::null_mut();

    thread = kmalloc(core::mem::size_of::<Thread>(), &M_THREAD, M_ZERO) as *mut Thread;
    if thread.is_null() {
        //return -ENOMEM;
        return -1;
    }

    (*thread).owner = proc;
    (*thread).tid = ((*proc).threads.count + 1) as isize;

    (*proc).threads.enqueue(thread);

    if !thread_ref.is_null() {
        *thread_ref = thread;
    }

    return 0;
}

pub unsafe fn thread_kill(thread: *mut Thread) -> isize {
    if thread.is_null() {
        //return -EINVAL;
        return -1;
    }

    /* free resources */
    arch_thread_kill(thread);
    (*thread).state = ThreadState::ZOMBIE;
    return 0;
}

pub unsafe fn thread_queue_sleep(queue: *mut Queue<*mut Thread>) -> isize {
    if queue.is_null() {
        panic!("sleeping in a blackhole?");
    }

    let sleep_node = (*queue).enqueue(curthread!());

    (*curthread!()).sleep_queue = queue;
    (*curthread!()).sleep_node  = sleep_node;
    (*curthread!()).state = ThreadState::ISLEEP;

    arch_sleep();

    /* Woke up */
    if ((*curthread!()).state != ThreadState::ISLEEP) {
        /* a signal interrupted the sleep */
        //return -EINTR;
        return -1; /* FIXME */
    } else {
        (*curthread!()).state = ThreadState::RUNNABLE;
        return 0;
    }
}

pub unsafe fn thread_queue_wakeup(queue: *mut Queue<*mut Thread>) -> isize {
    if queue.is_null() {
        //return -EINVAL;
        return -1;
    }

    while (*queue).count > 0 {
        let thread = (*queue).dequeue().unwrap();
        (*thread).sleep_node = core::ptr::null_mut();
        sched_thread_ready(thread);
    }

    return 0;
}

pub unsafe fn thread_create(thread: *mut Thread, stack: usize, entry: usize, uentry: usize, arg: usize, _attr: usize, new_thread: *mut *mut Thread) -> isize {
    let mut t: *mut Thread = core::ptr::null_mut();
    thread_new((*thread).owner, &mut t);

    arch_thread_create(t, stack, entry, uentry, arg);

    if !new_thread.is_null() {
        *new_thread = t;
    }

    return 0;
}
