use prelude::*;

use sys::sched::*;
use mm::*;

use arch::platform::pc::init::platform_timer_setup;
use arch::include::cpu::cpu::X86Regs;
use arch::cpu::gdt::x86_kernel_stack_set;
use arch::cpu::init::virtual_address;
use arch::include::core::arch::X86Thread;

extern "C" {
    fn x86_read_ip() -> usize;
    fn x86_goto(ip: usize, bp: usize, sp: usize) -> !;
    fn x86_sleep();
}

static mut TIMER_PERIOD: u32 = 0;
static mut TIMER_TICKS: u64 = 0;

pub unsafe fn arch_rtime_ns() -> u64 {
    return TIMER_TICKS * (TIMER_PERIOD as u64);
}

pub unsafe fn arch_rtime_us() -> u64 {
    return TIMER_TICKS * (TIMER_PERIOD as u64) / 1000;
}

pub unsafe fn arch_rtime_ms() -> u64 {
    return TIMER_TICKS * (TIMER_PERIOD as u64) / 1000000;
}

static FIRST_MEASURED_TIME: u64 = 0;

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

    TIMER_TICKS += 1;

    if kidle == 0 {
        let arch = (*curthread!()).arch as *mut X86Thread;


        let (ip, sp, bp);    

        llvm_asm!("mov %esp, $0":"=r"(sp)); /* read esp */
        llvm_asm!("mov %ebp, $0":"=r"(bp)); /* read ebp */
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
    TIMER_PERIOD = platform_timer_setup(2000000, x86_sched_handler);
}

unsafe fn __arch_idle() {
    loop {
        llvm_asm!("sti; hlt; cli;");
    }
}

static __IDLE_STACK: [u8; 8192] = [0; 8192];

pub unsafe fn arch_idle() {
    curthread!() = core::ptr::null_mut();

    let esp = virtual_address(0x100000usize) as *const u8 as usize;
    x86_kernel_stack_set(esp);
    let stack = &__IDLE_STACK as *const _ as usize + 8192;
    //extern void x86_goto(uintptr_t eip, uintptr_t ebp, uintptr_t esp) __attribute__((noreturn));
    x86_goto(__arch_idle as *const u8 as usize, stack, stack);
}

unsafe fn __arch_cur_thread_kill() {
    /* will set the stack to vma(0x100000) */
    (*curthread!()).kill();

    kfree(curthread!() as *mut u8);
    curthread!() = core::ptr::null_mut();
    kernel_idle();
}

pub unsafe fn arch_cur_thread_kill() {
    let stack = &__IDLE_STACK as *const _ as usize + 8192;

    //extern void x86_goto(uintptr_t eip, uintptr_t ebp, uintptr_t esp) __attribute__((noreturn));
    x86_goto(__arch_cur_thread_kill as usize, stack, stack);
}

pub unsafe fn arch_sleep() {
    x86_sleep();
}

