use prelude::*;

use arch::sys::*;
use core::fmt;
use sys::process::*;
use sys::sched::*;

malloc_define!(M_THREAD, "thread\0", "thread structure\0");

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ThreadState {
    RUNNABLE = 1,
    ISLEEP = 2, /* Interruptable SLEEP (I/O) */
    USLEEP = 3, /* Uninterruptable SLEEP (Waiting for event) */
    ZOMBIE = 4,
}

#[derive(Copy, Clone)]
pub struct ThreadStack {
    pub pointer: usize,
    pub base: usize,
    pub size: usize,
}

#[derive(Copy, Clone)]
pub struct Thread {
    /** thread id */
    pub tid: tid_t,

    /** thread current state */
    pub state: ThreadState,

    /** thread owner process */
    pub owner: *mut Process,

    /** thread stack */
    pub stack: ThreadStack,

    /** current sleep queue */
    pub sleep_queue: *mut Queue<*mut Thread>,
    pub sleep_node: *mut QueueNode<*mut Thread>,

    /** scheduler queue */
    pub sched_queue: *mut Queue<*mut Thread>,
    pub sched_node: *mut QueueNode<*mut Thread>,

    /** arch specific data */
    pub arch: *mut u8,

    /** thread flags */
    pub spawned: isize,
}

unsafe impl Sync for Thread {}

impl Thread {
    pub fn alloc() -> Box<Thread> {
        unsafe { Box::new_zeroed_tagged(&M_THREAD).assume_init() }
    }

    pub fn kill(&mut self) -> isize {
        unsafe {
            /* free resources */
            arch_thread_kill(self);
            self.state = ThreadState::ZOMBIE;
            return 0;
        }
    }
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

    while (*queue).count() > 0 {
        let thread = (*queue).dequeue().unwrap();
        (*thread).sleep_node = core::ptr::null_mut();
        sched_thread_ready(thread);
    }

    return 0;
}

pub unsafe fn thread_create(thread: *mut Thread, stack: usize, entry: usize, uentry: usize, arg: usize, _attr: usize, new_thread: *mut *mut Thread) -> isize {
    let mut t: *mut Thread = core::ptr::null_mut();
    (*(*thread).owner).new_thread(&mut t);

    arch_thread_create(t, stack, entry, uentry, arg);

    if !new_thread.is_null() {
        *new_thread = t;
    }

    return 0;
}
