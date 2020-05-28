use prelude::*;

use arch::include::core::arch::X86Thread;
use arch::include::cpu::cpu::*;
use mm::*;
use sys::sched::*;
use sys::thread::*;

malloc_define!(M_X86_FPU, "x86-fpu\0", "x86 FPU context\0");

#[repr(align(16))]
struct SaveBuffer([u8; 512]);

static mut FPU_CONTEXT: SaveBuffer = SaveBuffer([0; 512]);
pub static mut LAST_FPU_THREAD: *mut Thread = core::ptr::null_mut();

#[inline]
unsafe fn fpu_save() {
    llvm_asm!("fxsave ($0)"::"r"(&FPU_CONTEXT):"memory");
}

#[inline]
unsafe fn fpu_restore() {
    llvm_asm!("fxrstor ($0)"::"r"(&FPU_CONTEXT):"memory");
}

pub unsafe fn x86_fpu_enable() {
    llvm_asm!("clts");
    write_cr0((read_cr0() & !CR0_EM) | CR0_MP);
}

pub unsafe fn x86_fpu_disable() {
    write_cr0(read_cr0() | CR0_EM);
}

pub unsafe fn x86_fpu_init() {
    llvm_asm!("fninit");
}

pub unsafe fn x86_fpu_trap() {
    x86_fpu_enable();

    let arch: *mut X86Thread = (*curthread!()).arch as *mut X86Thread;

    if LAST_FPU_THREAD.is_null() {   /* Initialize */
        x86_fpu_init();
        (*arch).fpu_enabled = 1;
    } else if (curthread!() != LAST_FPU_THREAD) {
        let _arch: *mut X86Thread = (*LAST_FPU_THREAD).arch as *mut X86Thread;

        if (*_arch).fpu_context.is_null() {  /* Lazy allocate */
            (*_arch).fpu_context = kmalloc(512, &M_X86_FPU, 0);

            if (*_arch).fpu_context.is_null() {
                panic!("todo");
            }
        }

        fpu_save();
        core::ptr::copy(&FPU_CONTEXT as *const _ as *mut u8, (*_arch).fpu_context, 512);

        if (*arch).fpu_enabled != 0 {    /* Restore context */
            core::ptr::copy((*arch).fpu_context, &FPU_CONTEXT as *const _ as *mut u8, 512);
            fpu_restore();
        } else {
            x86_fpu_init();
            (*arch).fpu_enabled = 1;
        }
    }

    LAST_FPU_THREAD = curthread!();
}

