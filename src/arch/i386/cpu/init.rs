use prelude::*;

use mm::*;
use boot::*;
use kern::main::*;
use kern::kargs::*;

use arch::include::boot::multiboot::multiboot_info;
use arch::include::boot::boot::process_multiboot_info;
use arch::include::boot::multiboot::MultibootInfo;
use arch::platform::pc::init::platform_init;
use arch::earlycon::earlycon::earlycon_reinit;
use arch::cpu::isr::x86_isr_setup;
use arch::cpu::idt::x86_idt_setup;
use arch::cpu::gdt::x86_tss_setup;
use arch::cpu::gdt::x86_gdt_setup;
use arch::earlycon::earlycon::earlycon_init;

extern "C" {
    /* must be defined in linker script */
    static _VMA: u8;
}

#[no_mangle]
static mut __kboot: *mut BootInfo = core::ptr::null_mut();

pub unsafe fn virtual_address<T>(addr: usize) -> *mut T {
    (addr + &_VMA as *const _ as usize) as *const u8 as *mut T
}

pub unsafe fn local_address<T>(addr: usize) -> *mut T {
    (addr - &_VMA as *const _ as usize) as *const u8 as *mut T
}

#[no_mangle]
pub unsafe extern "C" fn x86_cpu_init() {
    earlycon_init();
    print!("x86: Welcome to AquilaOS!\n");

    print!("x86: installing GDT\n");
    x86_gdt_setup();
    x86_tss_setup(virtual_address::<u8>(0x100000usize) as usize);

    print!("x86: installing IDT\n");
    x86_idt_setup();

    print!("x86: installing ISRs\n");
    x86_isr_setup();

    print!("x86: processing multiboot info block at {:p}\n", multiboot_info as usize as *const u8);
    let boot = process_multiboot_info(multiboot_info as usize as *const MultibootInfo) as *mut BootInfo;
    __kboot = boot;

    //let boot = x86_process_multiboot_info();

    print!("x86: setting up memory manager\n");
    mm_setup(boot);

    print!("x86: setting up kernel allocator\n");
    kvmem_setup();

    /* parse command line */
    kargs_parse((*boot).cmdline);

    /* reinit early console */
    earlycon_reinit();

    platform_init();

    kmain(boot);
}
