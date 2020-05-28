use prelude::*;

use sys::session::*;
use sys::process::*;
use sys::thread::*;

use kern::string::*;
use arch::i386::sys::*;
use crate::include::core::arch::*;

use crate::{print};

pub static mut ready_queue: Queue<*mut Thread> = Queue::empty();

pub static mut _curthread: *mut Thread = core::ptr::null_mut();

#[macro_export]
macro_rules! curthread {
    () => {
        crate::sys::sched::_curthread
    }
}

#[macro_export]
macro_rules! curproc {
    () => {
        ((*crate::sys::sched::_curthread).owner)
    }
}

pub unsafe fn sched_thread_ready(thread: *mut Thread) {
    let sched_node = ready_queue.enqueue(thread);

    (*thread).sched_queue = &mut ready_queue;
    (*thread).sched_node  = sched_node;
}

#[no_mangle]
pub static mut kidle: isize = 0;

pub unsafe fn kernel_idle() {
    kidle = 1;
    arch_idle();
}

/* start thread execution */
pub unsafe fn sched_thread_spawn(thread: *mut Thread) {
    (*thread).spawned = 1;
    arch_thread_spawn(thread);
}

pub unsafe fn sched_init_spawn(init: *mut Process) {
    proc_init(init);

    /* init defaults */
    (*init).mask = 0775;
    (*init).cwd = strdup(b"/".as_ptr());

    arch_sched_init();

    session_new(init);

    //print!("sizeof(Thread) = {}\n", core::mem::size_of::<Thread>());

    curthread!() = (*(*init).threads.head).value as *mut Thread;
    (*curthread!()).state = ThreadState::RUNNABLE;

    //print!("{:?}\n", *curthread!());
    
    sched_thread_spawn(curthread!());
}

/* called from arch-specific timer event handler */
pub unsafe fn schedule() {
    if kidle == 0 {
        sched_thread_ready(curthread!());
    }

    kidle = 0;

    if ready_queue.count() == 0 {
        /* no ready threads, idle */
        kernel_idle();
    }

    curthread!() = ready_queue.dequeue().unwrap();
    (*curthread!()).sched_node = core::ptr::null_mut();

    if (*curthread!()).spawned != 0 {
        arch_thread_switch(curthread!());
    } else {
        sched_thread_spawn(curthread!());
    }
}

