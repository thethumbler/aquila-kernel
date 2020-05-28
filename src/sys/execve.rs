use prelude::*;

use kern::string::*;
use arch::i386::sys::*;
use crate::include::core::types::*;
use crate::include::core::arch::*;

use crate::sys::thread::*;
use crate::sys::binfmt::*;

use crate::include::mm::kvmem::*;

use crate::{malloc_declare};

malloc_declare!(M_BUFFER);

pub unsafe fn proc_execve(thread: *mut Thread, path: *const u8, argp: *const *const u8, envp: *const *const u8) -> isize {
    let proc = (*thread).owner;
    let u_argp = argp;
    let u_envp = envp;

    let mut argc = 0isize;
    let mut envc = 0isize;
    
    if !u_argp.is_null() {
        let mut arg_p = u_argp;
        while !(*arg_p).is_null() {
            argc += 1;
            arg_p = arg_p.offset(1);
        }
    }

    if !u_envp.is_null() {
        let mut env_p = u_envp;
        while !(*env_p).is_null() {
            envc += 1;
            env_p = env_p.offset(1);
        }
    }

    let mut argp = kmalloc(((argc + 1) as usize) * core::mem::size_of::<*const u8>(), &M_BUFFER, 0) as *mut *mut u8;
    let mut envp = kmalloc(((envc + 1) as usize) * core::mem::size_of::<*const u8>(), &M_BUFFER, 0) as *mut *mut u8;

    *argp.offset(argc) = core::ptr::null_mut();
    *envp.offset(envc) = core::ptr::null_mut();

    for i in 0..argc {
        *argp.offset(i) = strdup(*u_argp.offset(i));
    }

    for i in 0..envc {
        *envp.offset(i) = strdup(*u_envp.offset(i));
    }

    let mut err = 0;

    err = binfmt_load(proc, path, core::ptr::null_mut());
    if err != 0 {
        /* free used resources */
        for i in 0..=argc {
            kfree(*argp.offset(i));
        }

        kfree(argp as *mut u8);

        for i in 0..=envc {
            kfree(*envp.offset(i));
        }

        kfree(envp as *mut u8);

        return err;
    }

    (*thread).spawned = 0;
    
    arch_sys_execve(proc, (argc + 1) as usize, argp as *const *const u8, (envc + 1) as usize, envp as *const *const u8);
    core::ptr::write_bytes(&(*proc).sigaction as *const _ as *mut u8, 0, core::mem::size_of_val(&(*proc).sigaction));

    /* free used resources */
    for i in 0..argc {
        kfree(*argp.offset(i));
    }

    kfree(argp as *mut u8);

    for i in 0..envc {
        kfree(*envp.offset(i));
    }

    kfree(envp as *mut u8);
    
    return 0;
}

pub unsafe fn thread_execve(thread: *mut Thread, argp: *const *const u8, envp: *const *const u8) -> isize {
    let proc = (*thread).owner;

    /* start at the top of user stack */
    let mut stack = (*thread).stack.base;

    let mut argc = 0;
    let mut envc = 0;
    
    if !envp.is_null() {
        let mut env_p = envp;
        while !(*env_p).is_null() {
            envc += 1;
            env_p = env_p.offset(1);
        }
    }

    if !argp.is_null() {
        let mut arg_p = argp;
        while !(*arg_p).is_null() {
            argc += 1;
            arg_p = arg_p.offset(1);
        }
    }

    /* TODO remove M_ZERO */
    let u_envp = kmalloc(core::mem::size_of::<*mut u8>() * ((envc+1) as usize), &M_BUFFER, M_ZERO) as *mut *mut u8;
    let u_argp = kmalloc(core::mem::size_of::<*mut u8>() * ((envc+1) as usize), &M_BUFFER, M_ZERO) as *mut *mut u8;

    /* TODO support upward growing stacks */

    /* push envp strings */
    for i in (0..=envc - 1).rev() {
        stack -= strlen(*envp.offset(i)) + 1;
        strcpy(stack as *mut u8, *envp.offset(i));
        *u_envp.offset(i) = stack as *mut u8;
    }

    /* push argp strings */
    for i in (0..=argc - 1).rev() {
        stack -= strlen(*argp.offset(i)) + 1;
        strcpy(stack as *mut u8, *argp.offset(i));
        *u_argp.offset(i) = stack as *mut u8;
    }

    /* push envp array */
    stack -= ((envc+1) as usize) * core::mem::size_of::<*const u8>();
    memcpy(stack as *mut u8, u_envp as *mut u8, ((envc+1) as usize) * core::mem::size_of::<*const u8>());

    let env_ptr = stack;

    stack -= ((argc+1) as usize) * core::mem::size_of::<*const u8>();
    memcpy(stack as *mut u8, u_argp as *mut u8, ((argc+1) as usize) * core::mem::size_of::<*const u8>());

    let arg_ptr = stack;

    /* main(int argc, char **argv, char **envp) */
    stack -= core::mem::size_of::<usize>();
    *(stack as *mut usize) = env_ptr;

    stack -= core::mem::size_of::<usize>();
    *(stack as *mut usize) = arg_ptr;

    stack -= core::mem::size_of::<isize>();
    *(stack as *mut isize) = argc as isize;

    (*thread).stack.pointer = stack;

    kfree(u_envp as *mut u8);
    kfree(u_argp as *mut u8);
    
    return 0;
}
