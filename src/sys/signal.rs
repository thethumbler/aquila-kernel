use prelude::*;

use arch;

use crate::sys::proc::*;
use crate::sys::thread::*;
use crate::include::bits::errno::*;
use crate::include::core::types::*;

use crate::{curproc};

/* signal numbers */
pub const SIGHUP:   isize = 1;  /**< hangup */
pub const SIGINT:   isize = 2;  /**< interrupt */
pub const SIGQUIT:  isize = 3;  /**< quit */
pub const SIGILL:   isize = 4;  /**< illegal instruction (not reset when caught) */
pub const SIGTRAP:  isize = 5;  /**< trace trap (not reset when caught) */
pub const SIGIOT:   isize = 6;  /**< IOT instruction */
pub const SIGABRT:  isize = SIGIOT;  /**< used by abort, replace SIGIOT in the future */
pub const SIGEMT:   isize = 7;  /**< EMT instruction */
pub const SIGFPE:   isize = 8;  /**< floating point exception */
pub const SIGKILL:  isize = 9;  /**< kill (cannot be caught or ignored) */
pub const SIGBUS:   isize = 10; /**< bus error */
pub const SIGSEGV:  isize = 11; /**< segmentation violation */
pub const SIGSYS:   isize = 12; /**< bad argument to system call */
pub const SIGPIPE:  isize = 13; /**< write on a pipe with no one to read it */
pub const SIGALRM:  isize = 14; /**< alarm clock */
pub const SIGTERM:  isize = 15; /**< software termination signal from kill */
pub const SIGURG:   isize = 16; /**< urgent condition on IO channel */
pub const SIGSTOP:  isize = 17; /**< sendable stop signal not from tty */
pub const SIGTSTP:  isize = 18; /**< stop signal from tty */
pub const SIGCONT:  isize = 19; /**< continue a stopped process */
pub const SIGCHLD:  isize = 20; /**< to parent on child stop or exit */
pub const SIGCLD:   isize = SIGCHLD; /**< System V name for SIGCHLD */
pub const SIGTTIN:  isize = 21; /**< to readers pgrp upon background tty read */
pub const SIGTTOU:  isize = 22; /**< like TTIN for output if (tp->t_local&LTOSTOP) */
pub const SIGIO:    isize = 23; /**< input/output possible signal */
pub const SIGPOLL:  isize = SIGIO;   /**< System V name for SIGIO */
pub const SIGWINCH: isize = 24; /* window changed */
pub const SIGUSR1:  isize = 25; /**< user defined signal 1 */
pub const SIGUSR2:  isize = 26; /**< user defined signal 2 */

pub const SIG_MAX:  usize = 26;

pub const SIG_DFL:  usize = 0; /* Default action */

#[repr(C)]
#[derive(Copy, Clone)]
pub enum SignalDefaultAction {
    SIGACT_ABORT      = 1,
    SIGACT_TERMINATE  = 2,
    SIGACT_IGNORE     = 3,
    SIGACT_STOP       = 4,
    SIGACT_CONTINUE   = 5,
}

/**
 * \ingroup sys
 * \brief signal action
 */
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SignalAction {
    pub sa_handler: usize,
    pub sa_mask:    sigset_t,
    pub sa_flags:   isize,
}

#[no_mangle]
pub static sig_default_action: [SignalDefaultAction; SIG_MAX+1] = [
    /* invalid  */ SignalDefaultAction::SIGACT_IGNORE,
    /* SIGHUP   */ SignalDefaultAction::SIGACT_TERMINATE,
    /* SIGINT   */ SignalDefaultAction::SIGACT_TERMINATE,
    /* SIGQUIT  */ SignalDefaultAction::SIGACT_ABORT,
    /* SIGILL   */ SignalDefaultAction::SIGACT_ABORT,
    /* SIGTRAP  */ SignalDefaultAction::SIGACT_ABORT,
    /* SIGABRT  */ SignalDefaultAction::SIGACT_ABORT,
    /* SIGEMT   */ SignalDefaultAction::SIGACT_IGNORE,
    /* SIGFPE   */ SignalDefaultAction::SIGACT_ABORT,
    /* SIGKILL  */ SignalDefaultAction::SIGACT_TERMINATE,
    /* SIGBUS   */ SignalDefaultAction::SIGACT_ABORT,
    /* SIGSEGV  */ SignalDefaultAction::SIGACT_ABORT,
    /* SIGSYS   */ SignalDefaultAction::SIGACT_ABORT,
    /* SIGPIPE  */ SignalDefaultAction::SIGACT_TERMINATE,
    /* SIGALRM  */ SignalDefaultAction::SIGACT_TERMINATE,
    /* SIGTERM  */ SignalDefaultAction::SIGACT_TERMINATE,
    /* SIGURG   */ SignalDefaultAction::SIGACT_IGNORE,
    /* SIGSTOP  */ SignalDefaultAction::SIGACT_STOP,
    /* SIGTSTP  */ SignalDefaultAction::SIGACT_STOP,
    /* SIGCONT  */ SignalDefaultAction::SIGACT_CONTINUE,
    /* SIGCHLD  */ SignalDefaultAction::SIGACT_IGNORE,
    /* SIGTTIN  */ SignalDefaultAction::SIGACT_STOP,
    /* SIGTTOU  */ SignalDefaultAction::SIGACT_STOP,
    /* SIGIO    */ SignalDefaultAction::SIGACT_TERMINATE,
    /* SIGWINCH */ SignalDefaultAction::SIGACT_IGNORE,
    /* SIGUSR1  */ SignalDefaultAction::SIGACT_TERMINATE,
    /* SIGUSR2  */ SignalDefaultAction::SIGACT_TERMINATE,
];

pub unsafe fn signal_proc_send(proc: *mut Process, signal: isize) -> isize {
    if proc == curproc!() {
        arch::handle_signal(signal as usize);
    } else {
        (*proc).sig_queue.as_mut().unwrap().enqueue(signal);

        /* wake up main thread if sleeping - XXX */
        let thread = (*(*proc).threads.head).value as *mut Thread;

        if (*thread).state == ThreadState::ISLEEP {
            thread_queue_wakeup((*thread).sleep_queue);
        }
    }

    return 0;
}

pub unsafe fn signal_pgrp_send(pg: *mut ProcessGroup, signal: isize) -> isize {
    for qnode in (*pg).procs.as_mut().unwrap().iter() {
        let proc = (*qnode).value;
        signal_proc_send(proc, signal);
    }

    return 0;
}

pub unsafe fn signal_send(pid: pid_t, signal: isize) -> isize {
    if (*curproc!()).pid == pid {
        arch::handle_signal(signal as usize);
        return 0;
    } else {
        let proc = proc_pid_find(pid);

        if proc.is_null() {
            return -ESRCH;
        } else {
            return signal_proc_send(proc, signal);
        }
    }
}
