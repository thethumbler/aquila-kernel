use prelude::*;

use arch::i386::platform::pc::init::platform_timer_setup;
use crate::arch::i386::include::cpu::cpu::X86Regs;
use crate::include::core::types::TimeSpec;
use crate::arch::i386::cpu::gdt::x86_kernel_stack_set;
use crate::arch::i386::cpu::init::virtual_address;
use crate::arch::i386::include::core::arch::X86Thread;
use crate::include::mm::kvmem::*;
use crate::sys::sched::kernel_idle;
use crate::sys::thread::thread_kill;
use crate::sys::sched::schedule;
use crate::sys::sched::kidle;
use crate::{print, curthread};

extern "C" {
    fn x86_read_ip() -> usize;
    fn x86_goto(ip: usize, bp: usize, sp: usize) -> !;
    fn x86_sleep();
}

static mut timer_period: u32 = 0;
static mut timer_ticks: u64 = 0;

pub unsafe fn arch_rtime_ns() -> u64 {
    return timer_ticks * (timer_period as u64);
}

pub unsafe fn arch_rtime_us() -> u64 {
    return timer_ticks * (timer_period as u64) / 1000;
}

pub unsafe fn arch_rtime_ms() -> u64 {
    return timer_ticks * (timer_period as u64) / 1000000;
}

static first_measured_time: u64 = 0;

unsafe fn x86_sched_handler(r: *const X86Regs) {
    /* we check time every 2^16 ticks */
    /*
    if timer_ticks & 0xFFFF == 0 {
        if timer_ticks == 0 {
            let mut ts: TimeSpec;
            gettime(&mut ts);
            first_measured_time = ts.tv_sec;
        } else {
            let mut ts: TimeSpec;
            gettime(&mut ts);

            let measured_time = ts.tv_sec - first_measured_time;
            let calculated_time = arch_rtime_ms() / 1000;

            let delta = calculated_time - measured_time;

            if delta < -1 || delta > 1 {
                print!("warning: calculated time differs from measured time by {} seconds\n", delta);
                print!("calculated time: {} seconds\n", calculated_time);
                print!("measured time: {} seconds\n", measured_time);

                /* TODO: attempt to correct time */
            }
        }
    }
    */

    timer_ticks += 1;

    if kidle == 0 {
        let arch = (*curthread!()).arch as *mut X86Thread;


        let (ip, sp, bp);    

        asm!("mov %esp, $0":"=r"(sp)); /* read esp */
        asm!("mov %ebp, $0":"=r"(bp)); /* read ebp */
        ip = x86_read_ip();

        if (ip == -1isize as usize) {
            /* done switching */
            return;
        }

        (*arch).eip = ip;
        (*arch).esp = sp;
        (*arch).ebp = bp;
    }

    schedule();
}

pub unsafe fn arch_sched_init() {
    timer_period = platform_timer_setup(2000000, x86_sched_handler);
}

unsafe fn __arch_idle() {
    loop {
        asm!("sti; hlt; cli;");
    }
}

static __idle_stack: [u8; 8192] = [0; 8192];

pub unsafe fn arch_idle() {
    curthread!() = core::ptr::null_mut();

    let esp = virtual_address(0x100000usize) as *const u8 as usize;
    x86_kernel_stack_set(esp);
    let stack = &__idle_stack as *const _ as usize + 8192;
    //extern void x86_goto(uintptr_t eip, uintptr_t ebp, uintptr_t esp) __attribute__((noreturn));
    x86_goto(__arch_idle as *const u8 as usize, stack, stack);
}

unsafe fn __arch_cur_thread_kill() {
    thread_kill(curthread!());    /* Will set the stack to VMA(0x100000) */
    kfree(curthread!() as *mut u8);
    curthread!() = core::ptr::null_mut();
    kernel_idle();
}

pub unsafe fn arch_cur_thread_kill() {
    let stack = &__idle_stack as *const _ as usize + 8192;

    //extern void x86_goto(uintptr_t eip, uintptr_t ebp, uintptr_t esp) __attribute__((noreturn));
    x86_goto(__arch_cur_thread_kill as usize, stack, stack);
}

pub unsafe fn arch_sleep() {
    x86_sleep();
}

