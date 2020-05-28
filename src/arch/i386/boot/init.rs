use prelude::*;

use arch::i386::cpu::init::x86_cpu_init;
use crate::include::mm::mm::PAGE_SIZE;
use crate::arch::i386::cpu::init::virtual_address;
use crate::arch::i386::include::cpu::cpu::CR0_PG;
use crate::arch::i386::include::cpu::cpu::read_cr0;
use crate::arch::i386::include::cpu::cpu::write_cr0;
use crate::arch::i386::include::cpu::cpu::write_cr3;

const TABLE_SIZE: usize = 0x400 * PAGE_SIZE;
const TABLE_MASK: usize = TABLE_SIZE - 1;
const KERNEL_HEAP_SIZE: usize = 8 * 1024 * 1024;  /* 8 MiB */

#[repr(align(4096))]
struct PageTable([u32; 1024]);

#[link_section=".boot.bss"]
static mut _BSP_PD: PageTable = PageTable([0; 1024]);

extern "C" {
    static kernel_end: u8;
    fn early_init_fix_stack(offset: usize);
}

const P:   u32 = 1 << 0;
const RW:  u32 = 1 << 1;
const PCD: u32 = 1 << 4;

#[repr(align(4096))]
struct ScratchArea([u8; 1024 * 1024]);

#[link_section=".boot.bss"]
static mut scratch: ScratchArea = ScratchArea([0; 1024 * 1024]); /* 1 MiB scratch area */

#[inline]
#[link_section=".boot.text"]
unsafe fn enable_paging(page_directory: usize) {
    write_cr3(page_directory);
    let cr0 = read_cr0();
    write_cr0(cr0 | CR0_PG);
}

#[inline]
#[link_section=".boot.text"]
unsafe fn switch_to_higher_half() {
    /* zero out paging structure */
    for i in 0..1024 {
        core::ptr::write_volatile(&mut _BSP_PD.0[i], 0);
    }

    /* entries count required to map the kernel */
    let entries = (&kernel_end as *const u8 as usize + KERNEL_HEAP_SIZE + TABLE_MASK) / TABLE_SIZE;

    let mut _BSP_PT = &scratch as *const _ as *mut u32;

    /* identity map pages */
    for i in 0..entries * 1024 {
        *_BSP_PT.offset(i as isize) = (i * PAGE_SIZE) as u32 | P | RW;
    }

    /* map the lower-half */
    for i in 0..entries {
        _BSP_PD.0[i] = (_BSP_PT as *const u8 as u32 + (i * PAGE_SIZE) as u32) | P | RW;
    }

    /* map the upper-half */
    for i in 0..entries {
        _BSP_PD.0[768 + i] = (_BSP_PT as *const u8 as u32 + (i * PAGE_SIZE) as u32) | P | RW;
    }

    /* enable paging using bootstrap processor page directory */
    enable_paging(&_BSP_PD as *const _ as usize);
}

#[no_mangle]
#[link_section=".boot.text"]
pub unsafe extern "C" fn x86_bootstrap() {
    /* we assume that grub loaded a valid gdt */
    /* then we map the kernel to the higher half */
    switch_to_higher_half();

    /* now we make sp in the higher half */
    early_init_fix_stack(virtual_address(0) as *const u8 as usize);

    /* ready to get out of here */
    x86_cpu_init();

    /* why would we ever get back here? however we should be precautious */
    loop {
        asm!("hlt;");
    }
}

