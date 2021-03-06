use prelude::*;

use arch::cpu::gdt::x86_kernel_stack_set;
use arch::include::core::arch::X86_CS;
use arch::include::core::arch::X86_SS;
use arch::include::core::arch::X86Thread;
use arch::include::cpu::cpu::X86Regs;
use arch::sys::sched::arch_sleep;
use sys::process::*;
use sys::signal::*;
use sys::sched::*;

extern "C" {
    fn x86_jump_user(eax: usize, eip: usize, cs: usize, eflags: usize, esp: usize, ss: usize) -> !;
}

pub unsafe fn arch_handle_signal(sig: usize) {
    let mut handler = (*curproc!()).sigaction[sig].sa_handler;

    /* can't signal a zmobie */
    if (*curproc!()).running == 0 {
        return;
    }

    if handler as usize == SIG_DFL {
        handler = sig_default_action[sig] as usize;
    }

    let arch = (*curthread!()).arch as *mut X86Thread;

    match (handler) {
        /* SIGACT_IGNORE */ 3 => return,
        /* SIGACT_ABORT */ 1 |
        /* SIGACT_TERMINATE */ 2 => {
            (*curproc!()).exit = proc_exit!(sig, sig) as isize;
            proc_kill(curproc!());
            arch_sleep();
            /* unreachable */
        },
        _ => {}
    }

    (*arch).kstack -= core::mem::size_of::<X86Regs>();
    x86_kernel_stack_set((*arch).kstack);

    let mut sig_sp = (*((*arch).regs as *mut X86Regs)).esp;

    /* push signal number */
    sig_sp -= core::mem::size_of::<isize>();
    *(sig_sp as *mut isize) = sig as isize;

    /* push return address */
    sig_sp -= core::mem::size_of::<usize>();
    *(sig_sp as *mut usize) = 0x0FFF;

    x86_jump_user(0, handler, X86_CS, (*arch).eflags, sig_sp, X86_SS);
}

pub fn handle_signal(sig: usize) {
    unsafe { arch_handle_signal(sig) }
}
