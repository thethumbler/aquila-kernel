use prelude::*;

use crate::sys::syscall::syscall_table;
use crate::sys::syscall::syscall_cnt;
use crate::sys::thread::Thread;
use crate::arch::i386::include::core::arch::X86Thread;
use crate::arch::i386::include::cpu::cpu::X86Regs;
use crate::kern::print::cstr;
use crate::include::bits::errno::ENOSYS;
use crate::sys::syscall::Syscall;

use crate::{print, curthread, curproc};

#[no_mangle]
pub unsafe fn arch_syscall(r: *mut X86Regs) {
    if (*r).eax >= syscall_cnt {
        print!("[{}:{}] {}: undefined syscall {}\n", (*curproc!()).pid, (*curthread!()).tid, cstr((*curproc!()).name), (*r).eax);
        arch_syscall_return(curthread!(), -ENOSYS as usize);
        return;
    }
	
    let syscall = &syscall_table[(*r).eax].0 as *const _ as *const fn(usize, usize, usize);
    (*syscall)((*r).ebx, (*r).ecx, (*r).edx);
}

pub unsafe fn arch_syscall_return(thread: *mut Thread, val: usize) {
    let arch = (*thread).arch as *mut X86Thread;

    if (*thread).spawned != 0 {
        (*((*arch).regs as *mut X86Regs)).eax = val;
    } else {
        (*arch).eax = val;
    }
}

pub unsafe fn syscall_return(thread: *mut Thread, val: usize) {
    arch_syscall_return(thread, val);
}
