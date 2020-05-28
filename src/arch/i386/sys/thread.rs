use prelude::*;

use arch::cpu::fpu::*;
use arch::cpu::gdt::x86_kernel_stack_set;
use arch::cpu::init::virtual_address;
use arch::include::core::arch::*;
use arch::mm::i386::*;
use arch::sys::signal::arch_handle_signal;
use mm::*;
use sys::sched::*;
use sys::thread::*;

extern "C" {
    fn x86_read_ip() -> usize;
    fn x86_goto(ip: usize, bp: usize, sp: usize) -> !;
    fn x86_jump_user(eax: usize, eip: usize, cs: usize, eflags: usize, esp: usize, ss: usize) -> !;
}

malloc_define!(M_KERN_STACK, "kern-stack\0", "kernel stack\0");
malloc_define!(M_X86_THREAD, "x86-thread\0", "x86 thread structure\0");

macro push {
    ($stack:expr, $ty:ty, $value:expr) => {
        $stack -= core::mem::size_of::<$ty>();
        *($stack as *mut $ty) = $value as $ty;
    }
}

pub unsafe fn arch_thread_spawn(thread: *mut Thread) {
    let arch = (*thread).arch as *mut X86Thread;
    let pmap = (*(*thread).owner).vm_space.pmap;

    pmap_switch(pmap);
    x86_kernel_stack_set((*arch).kstack);
    x86_jump_user((*arch).eax, (*arch).eip, X86_CS, (*arch).eflags, (*arch).esp, X86_SS);
}

pub unsafe fn arch_thread_switch(thread: *mut Thread) {
    let arch = (*thread).arch as *mut X86Thread;
    let pmap = (*(*thread).owner).vm_space.pmap;

    pmap_switch(pmap);

    x86_kernel_stack_set((*arch).kstack);
    x86_fpu_disable();

    if (*(*thread).owner).sig_queue.as_ref().unwrap().count() != 0 {
        let sig = (*(*thread).owner).sig_queue.as_mut().unwrap().dequeue().unwrap();
        arch_handle_signal(sig as usize);
        /* if we get back here, the signal was ignored */
    }

    x86_goto((*arch).eip, (*arch).ebp, (*arch).esp);
}

pub unsafe fn arch_thread_create(thread: *mut Thread, stack: usize, entry: usize, uentry: usize, arg: usize) {
    let mut stack = stack;

    let arch = kmalloc(core::mem::size_of::<X86Thread>(), &M_X86_THREAD, M_ZERO) as *mut X86Thread;
    if arch.is_null() {
        panic!("todo");
    }

    (*arch).kstack = kmalloc(KERN_STACK_SIZE, &M_KERN_STACK, 0) as usize + KERN_STACK_SIZE;

    /* dummy return address */
    push!((*arch).kstack, usize, 0);

    /* dummy base pointer */
    push!((*arch).kstack, usize, 0);

    (*arch).eflags = X86_EFLAGS;
    (*arch).eip = entry;

    /* push thread argument */
    push!(stack, usize, arg);

    /* push user entry point */
    push!(stack, usize, uentry);

    /* dummy return address */
    push!(stack, usize, 0);

    (*arch).esp = stack;
    (*thread).arch = arch as *mut u8;
}

pub unsafe fn arch_thread_kill(thread: *mut Thread) {
    let arch = (*thread).arch as *mut X86Thread;

    if thread == curthread!() {
        /* we don't wanna die here */
        let esp = virtual_address(0x100000usize) as *const u8 as usize; /* XXX */
        x86_kernel_stack_set(esp);
    }

    if (*arch).kstack != 0 {
        kfree(((*arch).kstack - KERN_STACK_SIZE) as *mut u8);
    }

    if !(*arch).fpu_context.is_null() {
        kfree((*arch).fpu_context);
    }

    if LAST_FPU_THREAD == thread {
        LAST_FPU_THREAD = core::ptr::null_mut();
    }

    kfree(arch as *mut u8);
}

#[no_mangle]
pub unsafe extern "C" fn internal_arch_sleep() {
    let arch = (*curthread!()).arch as *mut X86Thread;
    //extern uintptr_t x86_read_ip(void);

    let ip;
    let sp;
    let bp;

    llvm_asm!("mov %esp, $0":"=r"(sp)); /* read esp */
    llvm_asm!("mov %ebp, $0":"=r"(bp)); /* read ebp */
    ip = x86_read_ip();

    if ip == -1isize as usize {
        /* done switching */
        return;
    }

    (*arch).eip = ip;
    (*arch).esp = sp;
    (*arch).ebp = bp;
    kernel_idle();
}
