use prelude::*;

use arch::i386::sys::syscall::arch_syscall;
use arch::i386::cpu::idt::x86_idt_gate_user_set;
use arch::i386::cpu::idt::x86_idt_gate_set;
use arch::i386::cpu::fpu::x86_fpu_trap;
use arch::i386::mm::i386::arch_mm_page_fault;
use crate::arch::i386::include::cpu::cpu::*;
use crate::arch::i386::include::core::arch::*;
use crate::sys::sched::*;
use crate::arch::i386::cpu::gdt::x86_kernel_stack_set;

use crate::{print, curthread};

extern "Rust" {
    fn __x86_isr0();
    fn __x86_isr1();
    fn __x86_isr2();
    fn __x86_isr3();
    fn __x86_isr4();
    fn __x86_isr5();
    fn __x86_isr6();
    fn __x86_isr7();
    fn __x86_isr8();
    fn __x86_isr9();
    fn __x86_isr10();
    fn __x86_isr11();
    fn __x86_isr12();
    fn __x86_isr13();
    fn __x86_isr14();
    fn __x86_isr15();
    fn __x86_isr16();
    fn __x86_isr17();
    fn __x86_isr18();
    fn __x86_isr19();
    fn __x86_isr20();
    fn __x86_isr21();
    fn __x86_isr22();
    fn __x86_isr23();
    fn __x86_isr24();
    fn __x86_isr25();
    fn __x86_isr26();
    fn __x86_isr27();
    fn __x86_isr28();
    fn __x86_isr29();
    fn __x86_isr30();
    fn __x86_isr31();
    fn __x86_isr128();
}

extern "C" {
    static __x86_isr_int_num: u32;
    static __x86_isr_err_num: u32;
}

/* Refer to 
 * - Intel 64 and IA-32 Architectures Software Developer’s Manual
 * - Volume 3: System Programming Guide
 * - Table 6-1. Protected-Mode Exceptions and Interrupts
 */

static int_msg: [&str; 32] = [
    /* 0x00 */ "#DE: Divide Error",
    /* 0x01 */ "#DB: Debug Exception",
    /* 0x02 */ "NMI Interrupt",
    /* 0x03 */ "#BP: Breakpoint",
    /* 0x04 */ "#OF: Overflow",
    /* 0x05 */ "#BR: BOUND Range Exceeded",
    /* 0x06 */ "#UD: Invalid Opcode (Undefined Opcode)",
    /* 0x07 */ "#NM: Device Not Available (No Math Coprocessor)",
    /* 0x08 */ "#DF: Double Fault",
    /* 0x09 */ "Coprocessor Segment Overrun (reserved)",
    /* 0x0a */ "#TS: Invalid TSS",
    /* 0x0b */ "#NP: Segment Not Present",
    /* 0x0C */ "#SS: Stack-Segment Fault",
    /* 0x0D */ "#GP: General Protection",
    /* 0x0E */ "#PF: Page Fault",
    /* 0x0F */ "Reserved",
    /* 0x10 */ "#MF: x87 FPU Floating-Point Error (Math Fault)",
    /* 0x11 */ "#AC: Alignment Check",
    /* 0x12 */ "#MC: Machine Check",
    /* 0x13 */ "#XM: SIMD Floating-Point Exception",
    /* 0x14 */ "#VE: Virtualization Exception",
    /* 0x15 */ "Reserved",
    /* 0x16 */ "Reserved",
    /* 0x17 */ "Reserved",
    /* 0x18 */ "Reserved",
    /* 0x19 */ "Reserved",
    /* 0x1A */ "Reserved",
    /* 0x1B */ "Reserved",
    /* 0x1C */ "Reserved",
    /* 0x1D */ "Reserved",
    /* 0x1E */ "Reserved",
    /* 0x1F */ "Reserved"
];

#[no_mangle]
pub unsafe extern "C" fn __x86_isr(regs: *mut X86Regs) {
    if __x86_isr_int_num == 0xE { /* Page Fault */
        if curthread!().is_null() {
            panic!("page fault inside the kernel!");
        }

        let arch: *mut X86Thread = (*curthread!()).arch as *mut X86Thread;
        //arch->regs = regs;

        if (*regs).eip == 0x0FFF {  /* Signal return */

            /* Fix kstack and regs pointers*/
            //(*arch).regs    = (*arch).kstack;
            //(*arch).kstack += core::mem::size_of::<X86Regs>(); 
            //x86_kernel_stack_set((*arch).kstack);

            panic!("todo");
            //extern void return_from_signal(uintptr_t) __attribute__((noreturn));
            //return_from_signal((uintptr_t) arch->regs);
        }

        let addr = read_cr2();
        arch_mm_page_fault(addr, __x86_isr_err_num as usize);
        return;
    }

    if (__x86_isr_int_num == 0x07) {  /* FPU Trap */
        x86_fpu_trap();
        return;
    }
    
    if (__x86_isr_int_num == 0x80) {  /* syscall */
        let arch: *mut X86Thread = (*curthread!()).arch as *mut X86Thread;
        (*arch).regs = regs as *mut u8;
        //asm volatile ("sti");
        arch_syscall(regs);
        return;
    }


    if (__x86_isr_int_num < 32) {
        let msg = int_msg[__x86_isr_int_num as usize];
        print!("Recieved interrupt {} [err={}]: {}\n", __x86_isr_int_num, __x86_isr_err_num, msg);

        if (__x86_isr_int_num == 0x0E) { /* Page Fault */
            print!("CR2 = {:p}\n", read_cr2() as *const u8);
        }

        //x86_dump_registers(regs);
        panic!("Kernel Exception");
    } else {
        print!("Unhandled interrupt: {}\n", __x86_isr_int_num);
        panic!("Kernel Exception");
    }
}

pub unsafe fn x86_isr_setup() {   
    x86_idt_gate_set(0x00, __x86_isr0  as usize);
    x86_idt_gate_set(0x01, __x86_isr1  as usize);
    x86_idt_gate_set(0x02, __x86_isr2  as usize);
    x86_idt_gate_set(0x03, __x86_isr3  as usize);
    x86_idt_gate_set(0x04, __x86_isr4  as usize);
    x86_idt_gate_set(0x05, __x86_isr5  as usize);
    x86_idt_gate_set(0x06, __x86_isr6  as usize);
    x86_idt_gate_set(0x07, __x86_isr7  as usize);
    x86_idt_gate_set(0x08, __x86_isr8  as usize);
    x86_idt_gate_set(0x09, __x86_isr9  as usize);
    x86_idt_gate_set(0x0A, __x86_isr10 as usize);
    x86_idt_gate_set(0x0B, __x86_isr11 as usize);
    x86_idt_gate_set(0x0C, __x86_isr12 as usize);
    x86_idt_gate_set(0x0D, __x86_isr13 as usize);
    x86_idt_gate_set(0x0E, __x86_isr14 as usize);
    x86_idt_gate_set(0x0F, __x86_isr15 as usize);
    x86_idt_gate_set(0x10, __x86_isr16 as usize);
    x86_idt_gate_set(0x11, __x86_isr17 as usize);
    x86_idt_gate_set(0x12, __x86_isr18 as usize);
    x86_idt_gate_set(0x13, __x86_isr19 as usize);
    x86_idt_gate_set(0x14, __x86_isr20 as usize);
    x86_idt_gate_set(0x15, __x86_isr21 as usize);
    x86_idt_gate_set(0x16, __x86_isr22 as usize);
    x86_idt_gate_set(0x17, __x86_isr23 as usize);
    x86_idt_gate_set(0x18, __x86_isr24 as usize);
    x86_idt_gate_set(0x19, __x86_isr25 as usize);
    x86_idt_gate_set(0x1A, __x86_isr26 as usize);
    x86_idt_gate_set(0x1B, __x86_isr27 as usize);
    x86_idt_gate_set(0x1C, __x86_isr28 as usize);
    x86_idt_gate_set(0x1D, __x86_isr29 as usize);
    x86_idt_gate_set(0x1E, __x86_isr30 as usize);
    x86_idt_gate_set(0x1F, __x86_isr31 as usize);
    x86_idt_gate_user_set(0x80, __x86_isr128 as usize);
}

