use prelude::*;

use sys::session::*;
use sys::process::*;
use sys::thread::*;
use arch::sys::*;

pub static mut READY_QUEUE: Queue<*mut Thread> = Queue::empty();
pub static mut _CURTHREAD: *mut Thread = core::ptr::null_mut();

pub macro curthread {
    () => {
        crate::sys::sched::_CURTHREAD
    }
}

pub macro curproc {
    () => {
        ((*crate::sys::sched::_CURTHREAD).owner)
    }
}

pub unsafe fn sched_thread_ready(thread: *mut Thread) {
    let sched_node = READY_QUEUE.enqueue(thread);

    (*thread).sched_queue = &mut READY_QUEUE;
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

    curthread!() = (*init).threads.head().unwrap().value;
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

    if READY_QUEUE.count() == 0 {
        /* no ready threads, idle */
        kernel_idle();
    }

    curthread!() = READY_QUEUE.dequeue().unwrap();
    (*curthread!()).sched_node = core::ptr::null_mut();

    if (*curthread!()).spawned != 0 {
        arch_thread_switch(curthread!());
    } else {
        sched_thread_spawn(curthread!());
    }
}

