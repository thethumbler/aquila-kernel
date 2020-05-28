use prelude::*;

use crate::include::mm::kvmem::*;
use crate::arch::i386::include::cpu::cpu::*;
use crate::arch::i386::include::core::arch::X86Thread;
use crate::sys::thread::Thread;
use crate::sys::sched::_curthread;
use crate::{curthread, malloc_define};

malloc_define!(M_X86_FPU, "x86-fpu\0", "x86 FPU context\0");

#[repr(align(16))]
struct SaveBuffer([u8; 512]);

static mut fpu_context: SaveBuffer = SaveBuffer([0; 512]);

#[no_mangle]
pub static mut last_fpu_thread: *mut Thread = core::ptr::null_mut();

#[inline]
unsafe fn fpu_save() {
    asm!("fxsave ($0)"::"r"(&fpu_context):"memory");
}

#[inline]
unsafe fn fpu_restore() {
    asm!("fxrstor ($0)"::"r"(&fpu_context):"memory");
}

pub unsafe fn x86_fpu_enable() {
    asm!("clts");
    write_cr0((read_cr0() & !CR0_EM) | CR0_MP);
}

pub unsafe fn x86_fpu_disable() {
    write_cr0(read_cr0() | CR0_EM);
}

pub unsafe fn x86_fpu_init() {
    asm!("fninit");
}

pub unsafe fn x86_fpu_trap() {
    x86_fpu_enable();

    let arch: *mut X86Thread = (*curthread!()).arch as *mut X86Thread;

    if last_fpu_thread.is_null() {   /* Initialize */
        x86_fpu_init();
        (*arch).fpu_enabled = 1;
    } else if (curthread!() != last_fpu_thread) {
        let _arch: *mut X86Thread = (*last_fpu_thread).arch as *mut X86Thread;

        if (*_arch).fpu_context.is_null() {  /* Lazy allocate */
            (*_arch).fpu_context = kmalloc(512, &M_X86_FPU, 0);

            if (*_arch).fpu_context.is_null() {
                panic!("todo");
            }
        }

        fpu_save();
        core::ptr::copy(&fpu_context as *const _ as *mut u8, (*_arch).fpu_context, 512);

        if (*arch).fpu_enabled != 0 {    /* Restore context */
            core::ptr::copy((*arch).fpu_context, &fpu_context as *const _ as *mut u8, 512);
            fpu_restore();
        } else {
            x86_fpu_init();
            (*arch).fpu_enabled = 1;
        }
    }

    last_fpu_thread = curthread!();
}

