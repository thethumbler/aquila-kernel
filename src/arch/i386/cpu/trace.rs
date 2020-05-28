use prelude::*;

use sys::binfmt::elf::*;
use boot::BootInfo;
use kern::print::cstr;
use crate::{print};

struct StackFrame {
    bp: *const StackFrame,
    ip: usize,
}

extern "C" {
    static __kboot: *const BootInfo;
}

#[no_mangle]
static mut __printing_trace: usize = 0;

pub unsafe fn arch_stack_trace() {
    __printing_trace = 1;

    let mut stk: *const StackFrame;

    llvm_asm!("movl %ebp, $0":"=r"(stk));

    print!("stack trace:\n");

    let strs = (*(*__kboot).strtab).sh_addr as *const u8;

    let mut frame = 0;

    while !stk.is_null() && frame < 20 {
        /* find symbol */
        let mut sym: *const Elf32Symbol = (*(*__kboot).symtab).sh_addr as *const Elf32Symbol;

        let mut found = false;

        for i in 0..(*__kboot).symnum {
            if (elf32_st_type((*sym).st_info) == STT_FUNC) {
                if (*stk).ip > (*sym).st_value as usize && (*stk).ip < ((*sym).st_value + (*sym).st_size) as usize {
                    found = true;
                    break;
                }
            }

            sym = sym.offset(1);
        }

        if (found) {
            let off = (*stk).ip - (*sym).st_value as usize;
            let symname = cstr(strs.offset((*sym).st_name as isize));
            print!("  [{:p}] {}+{:#x}\n", (*stk).ip as *const u8, symname, off);
        } else {
            print!("  [{:p}]\n", (*stk).ip as *const u8);
        }

        stk = (*stk).bp;
    }
}

