use prelude::*;
use mm::*;

use crate::{malloc_declare};

pub const X86_SS:     usize = 0x20 | 3;
pub const X86_EFLAGS: usize = 0x200;
pub const X86_CS:     usize = 0x18 | 3;

#[repr(C)]
pub struct X86Thread {
    pub kstack: usize, /* Kernel stack */

    pub eip: usize,
    pub esp: usize,
    pub ebp: usize,
    pub eflags: usize,
    pub eax: usize,    /* For syscall return if thread is not spawned */

    //struct x86_regs *regs;  /* Pointer to registers on the stack */
    pub regs: *mut u8,
    pub fpu_context: *mut u8,

    /* Flags */
    pub fpu_enabled: isize,
}

//void arch_syscall(struct x86_regs *r);

//#define USER_STACK      (0xC0000000UL)
//#define USER_STACK_BASE (USER_STACK - USER_STACK_SIZE)
//
pub const KERN_STACK_SIZE: usize = 8192usize;  /* 8 KiB */
//
//static inline void arch_interrupts_enable(void)
//{
//    asm volatile ("sti");
//}
//
//static inline void arch_interrupts_disable(void)
//{
//    asm volatile ("cli");
//}
//
//void x86_jump_user(uintptr_t eax, uintptr_t eip, uintptr_t cs, uintptr_t eflags, uintptr_t esp, uintptr_t ss) __attribute__((noreturn));
//void x86_goto(uintptr_t eip, uintptr_t ebp, uintptr_t esp) __attribute__((noreturn));
//
//#include_next <core/arch.h>
//
//#endif /* ! _X86_ARCH_H */

