use prelude::*;

use mm::*;
use sys::process::*;
use sys::binfmt::*;
use sys::thread::*;
use sys::sched::*;

use arch::mm::i386::*;
use arch::sys::execve::arch_sys_execve;
use arch::include::core::arch::KERN_STACK_SIZE;
use arch::include::core::arch::X86Thread;
use arch::include::core::arch::X86_EFLAGS;

malloc_declare!(M_X86_THREAD);
malloc_declare!(M_KERN_STACK);

pub unsafe fn arch_proc_init(proc: *mut Process) {
    let arch: *mut X86Thread = kmalloc(core::mem::size_of::<X86Thread>(), &M_X86_THREAD, M_ZERO) as *mut X86Thread;
    if arch.is_null() {
        panic!("todo");
    }

    let kstack_base = kmalloc(KERN_STACK_SIZE, &M_KERN_STACK, 0) as usize;

    (*arch).kstack = kstack_base + KERN_STACK_SIZE;   /* Kernel stack */
    (*arch).eip    = (*proc).entry;
    (*arch).esp    = USER_STACK;
    (*arch).eflags = X86_EFLAGS;

    let thread = (*proc).threads.head().unwrap().value;
    (*thread).arch = arch as *mut u8;
}

pub unsafe fn arch_init_execve(proc: *mut Process, argc: isize, argp: *const *const u8, envc: isize, envp: *const *const u8) {
    let pmap = (*proc).vm_space.pmap;
    let thread = (*proc).threads.head().unwrap().value;

    curthread!() = thread;
    pmap_switch(pmap);

    arch_sys_execve(proc, argc as usize, argp, envc as usize, envp);
    curthread!() = core::ptr::null_mut();
}
