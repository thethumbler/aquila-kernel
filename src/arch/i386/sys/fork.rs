use prelude::*;

use sys::process::*;
use sys::thread::*;
//use crate::arch::Arch;
//use crate::mm::kvmem::*;
//use crate::arch::*;
use crate::arch::i386::include::core::*;
use crate::arch::i386::include::cpu::*;

use crate::include::mm::kvmem::*;
use crate::{malloc_declare};

malloc_declare!(M_X86_THREAD);
malloc_declare!(M_KERN_STACK);

extern "C" {
    fn x86_fork_return();
}

pub unsafe fn arch_proc_fork(thread: *mut Thread, fork: *mut Process) -> isize {
    let mut err = 0;

    let mut ptarch: *mut X86Thread = (*thread).arch as *mut X86Thread;
    let mut ftarch: *mut X86Thread = core::ptr::null_mut();

    ftarch = kmalloc(core::mem::size_of::<X86Thread>(), &M_X86_THREAD, 0) as *mut X86Thread;

    if ftarch.is_null() {
        /* Failed to allocate fork thread arch structure */
        //err = -ENOMEM;
        //goto free_resources;
        panic!("todo");
    }

    /* Setup kstack */
    let fkstack_base: usize = kmalloc(KERN_STACK_SIZE, &M_KERN_STACK, 0) as *const _ as usize;
    (*ftarch).kstack = fkstack_base + KERN_STACK_SIZE;

    /* Copy registers */
    let fork_regs: *mut X86Regs = ((*ftarch).kstack - ((*ptarch).kstack - (*ptarch).regs as usize)) as *mut X86Regs;
    (*ftarch).regs = fork_regs as *mut u8;

    /* Copy kstack */
    core::ptr::copy(((*ptarch).kstack - KERN_STACK_SIZE) as *const u8, fkstack_base as *mut u8, KERN_STACK_SIZE);


    (*ftarch).eip = x86_fork_return as *const u8 as usize;
    (*ftarch).esp = fork_regs as *const u8 as usize;

    let fthread: *mut Thread = (*(*fork).threads.head).value;
    (*fthread).arch = ftarch as *mut u8;

    (*ftarch).fpu_enabled = 0;
    (*ftarch).fpu_context = core::ptr::null_mut();

    return 0;

    /*
free_resources:
    if (ftarch)
        kfree(ftarch);
    return err;
    */
}

pub fn proc_fork(thread: *mut Thread, proc: *mut Process) -> isize {
    unsafe { arch_proc_fork(thread, proc) }
}
