use prelude::*;

use sys::process::Process;
use sys::thread::*;
use sys::binfmt::*;
use mm::*;

use crate::arch::i386::include::core::arch::X86_EFLAGS;
use crate::arch::i386::include::core::arch::X86Thread;
use crate::arch::i386::include::cpu::cpu::{read_cr3, write_cr3};
use crate::{malloc_declare};

malloc_declare!(M_BUFFER);

pub unsafe fn tlb_flush() {
    write_cr3(read_cr3());
}

pub unsafe fn arch_sys_execve(proc: *mut Process, argc: usize, _argp: *const *const u8, envc: usize, _envp: *const *const u8) {
    let thread = (*(*proc).threads.head).value as *mut Thread;
    let arch = (*thread).arch as *mut X86Thread;

    (*arch).eip = (*proc).entry;
    (*arch).eflags = X86_EFLAGS;

    let argp = _argp;
    let u_argp = kmalloc(argc * core::mem::size_of::<usize>(), &M_BUFFER, 0) as *mut *mut u8;

    let envp = _envp;
    let u_envp = kmalloc(envc * core::mem::size_of::<usize>(), &M_BUFFER, 0) as *mut *mut u8;

    /* start at the top of user stack */
    let mut stack: usize = USER_STACK;
    tlb_flush();

    /* push envp strings */
    let mut tmp_envc = envc - 1;
    *u_envp.offset(tmp_envc as isize) = core::ptr::null_mut();

    for i in (0..envc-1).rev() {
        stack -= strlen(*envp.offset(i as isize)) + 1;
        strcpy(stack as *mut u8, *envp.offset(i as isize));
        tmp_envc -= 1;
        *u_envp.offset(tmp_envc as isize) = stack as *mut u8;
    }

    /* push argp strings */
    let mut tmp_argc = argc - 1;
    *u_argp.offset(tmp_argc as isize) = core::ptr::null_mut();

    for i in (0..argc-1).rev() {
        stack -= strlen(*argp.offset(i as isize)) + 1;
        strcpy(stack as *mut u8, *argp.offset(i as isize));
        tmp_argc -= 1;
        *u_argp.offset(tmp_argc as isize) = stack as *mut u8;
    }

    stack -= envc * core::mem::size_of::<usize>();
    memcpy(stack as *mut u8, u_envp as *const u8, envc * core::mem::size_of::<usize>());

    let env_ptr = stack as usize;

    stack -= argc * core::mem::size_of::<usize>();
    memcpy(stack as *mut u8, u_argp as *const u8, argc * core::mem::size_of::<usize>());

    let arg_ptr = stack as usize;

    /* main(int argc, char **argv, char **envp) */
    stack -= core::mem::size_of::<usize>();
    *(stack as *mut usize) = env_ptr;

    stack -= core::mem::size_of::<usize>();
    *(stack as *mut usize) = arg_ptr;

    stack -= core::mem::size_of::<usize>();
    *(stack as *mut usize) = argc - 1;

    kfree(u_envp as *mut u8);
    kfree(u_argp as *mut u8);

    (*arch).esp = stack as usize;
}

