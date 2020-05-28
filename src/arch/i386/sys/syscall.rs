use prelude::*;

use arch::include::core::arch::X86Thread;
use arch::include::cpu::cpu::X86Regs;
use kern::print::cstr;
use sys::syscall::*;
use sys::thread::*;
use sys::sched::*;

#[no_mangle]
pub unsafe fn arch_syscall(r: *mut X86Regs) {
    if (*r).eax >= SYSCALL_CNT {
        print!("[{}:{}] {}: undefined syscall {}\n", (*curproc!()).pid, (*curthread!()).tid, cstr((*curproc!()).name), (*r).eax);
        arch_syscall_return(curthread!(), -ENOSYS as usize);
        return;
    }
	
    let syscall = &SYSCALL_TABLE[(*r).eax].0 as *const _ as *const fn(usize, usize, usize);
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
