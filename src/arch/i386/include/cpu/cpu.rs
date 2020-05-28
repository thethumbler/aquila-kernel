use prelude::*;

#[repr(C)]
pub struct X86Regs {
    pub edi: usize,
    pub esi: usize,
    pub ebp: usize,
    pub ebx: usize,
    pub ecx: usize,
    pub edx: usize,
    pub eax: usize,
    pub eip: usize,
    pub cs: usize,
    pub eflags: usize,
    pub esp: usize,
    pub ss: usize,
}

/*
static inline void x86_dump_registers(struct x86_regs *regs)
{
    printk("Registers dump:\n");
#if ARCH_BITS==32
    printk("edi = %p\n", regs->edi);
    printk("esi = %p\n", regs->esi);
    printk("ebp = %p\n", regs->ebp);
    printk("ebx = %p\n", regs->ebx);
    printk("ecx = %p\n", regs->ecx);
    printk("edx = %p\n", regs->edx);
    printk("eax = %p\n", regs->eax);
    printk("eip = %p\n", regs->eip);
    printk("cs  = %p\n", regs->cs );
    printk("eflags = %p\n", regs->eflags);
    printk("esp = %p\n", regs->esp);
    printk("ss  = %p\n", regs->ss);
#else
    printk("r15 = %p\n", regs->r15);
    printk("r14 = %p\n", regs->r14);
    printk("r13 = %p\n", regs->r13);
    printk("r12 = %p\n", regs->r12);
    printk("r11 = %p\n", regs->r11);
    printk("r10 = %p\n", regs->r10);
    printk("r9  = %p\n", regs->r9);
    printk("r8  = %p\n", regs->r8);
    printk("rdi = %p\n", regs->rdi);
    printk("rsi = %p\n", regs->rsi);
    printk("rbp = %p\n", regs->rbp);
    printk("rbx = %p\n", regs->rbx);
    printk("rcx = %p\n", regs->rcx);
    printk("rdx = %p\n", regs->rdx);
    printk("rax = %p\n", regs->rax);
    printk("rip = %p\n", regs->rip);
    printk("cs  = %p\n", regs->cs );
    printk("rflags = %p\n", regs->rflags);
    printk("rsp = %p\n", regs->rsp);
    printk("ss  = %p\n", regs->ss);
#endif
}

struct x86_cpu {
    int id;
    union  x86_cpuid_vendor   vendor;
    //struct x86_cpuid_features features;
};
*/

/* CR0 */
pub const CR0_PG: usize = 1 << 31;
pub const CR0_MP: usize = 1 << 1;
pub const CR0_EM: usize = 1 << 2;
pub const CR0_NE: usize = 1 << 5;

/* CR4 */
//#define CR4_PSE _BV(4)

/* CPU function */
#[inline]
pub unsafe fn read_cr0() -> usize {
    let retval;
    asm!("mov %cr0, $0":"=r"(retval));
    retval
}

#[inline]
pub unsafe fn read_cr1() -> usize {
    let retval;
    asm!("mov %cr1, $0":"=r"(retval));
    retval
}

#[inline]
pub unsafe fn read_cr2() -> usize {
    let retval;
    asm!("mov %cr2, $0":"=r"(retval));
    retval
}

#[inline]
pub unsafe fn read_cr3() -> usize {
    let retval;
    asm!("mov %cr3, $0":"=r"(retval));
    retval
}

#[inline]
pub unsafe fn read_cr4() -> usize {
    let retval;
    asm!("mov %cr4, $0":"=r"(retval));
    retval
}

#[inline]
pub unsafe fn write_cr0(val: usize) {
    asm!("mov $0, %cr0"::"r"(val));
}

#[inline]
pub unsafe fn write_cr1(val: usize) {
    asm!("mov $0, %cr1"::"r"(val));
}

#[inline]
pub unsafe fn write_cr2(val: usize) {
    asm!("mov $0, %cr2"::"r"(val));
}

#[inline]
pub unsafe fn write_cr3(val: usize) {
    asm!("mov $0, %cr3"::"r"(val));
}

#[inline]
pub unsafe fn write_cr4(val: usize) {
    asm!("mov $0, %cr4"::"r"(val));
}

/*
/* cpu/gdt.c */
void x86_gdt_setup(void);
void x86_tss_setup(uintptr_t sp);
void x86_kernel_stack_set(uintptr_t sp);

/* cpu/idt.c */
void x86_idt_setup(void);
void x86_idt_gate_set(uint32_t id, uintptr_t offset);
void x86_idt_gate_user_set(uint32_t id, uintptr_t offset);

/* cpu/isr.c */
void x86_isr_setup(void);

/* cpu/fpu.c */
void x86_fpu_enable(void);
void x86_fpu_disable(void);
void x86_fpu_trap(void);

//void pic_setup(void);
//void pic_disable(void);
//void pit_setup(uint32_t);
//void acpi_setup(void);
//uintptr_t acpi_rsdt_find(char signature[4]);
//void hpet_setup(void);
//int hpet_timer_setup(size_t period_ns, void (*handler)());

//#include "msr.h"
//#include "sdt.h"
//#include "pit.h"

#include_next <cpu/cpu.h>

#endif /* ! _X86_CPU_H */
*/
