use prelude::*;

use mm::*;
use fs::{self, *};
use fs::initramfs::*;
use sys::process::*;
use sys::sched::*;
use sys::binfmt::*;
use sys::thread::*;
use dev::kdev::*;
use kern::module;
use boot::*;

use arch::sys::proc::arch_proc_init;
use arch::sys::proc::arch_init_execve;
use arch::mm::i386::pmap_switch;

pub unsafe fn kmain(boot: *const BootInfo) {
    /* FIXME */
    /* insert a dummy entry at last allocatable address */
    let mut dummy = VmEntry {
        base: 0xFF000000,
        size: PAGE_SIZE,

        ..VmEntry::none()
    };

    dummy.qnode = kvm_space.vm_entries.enqueue(&mut dummy);

    kdev_init();
    vfs_init();
    module::init();

    if (*boot).modules_count != 0 {
        if let Err(err) = load_ramdisk((*boot).modules as *mut u8) {
            panic!("failed to load ramdisk: {:?}", err);
        }
    } else {
        panic!("no modules loaded: unable to load ramdisk");
    }

    print!("kernel: loading init process\n");

    let mut init: *mut Process = core::ptr::null_mut();
    let mut err = 0;

    err = proc_new(&mut init);
    if err != 0 {
        panic!("failed to allocate process structure for init");
    }

    curthread!() = (*init).threads.head().unwrap().value;
    curproc!()   = init;

    pmap_switch((*init).vm_space.pmap);

    let init_p = b"/init\0".as_ptr();

    err = binfmt_load(init, init_p, &mut init);
    if err != 0 {
        print!("kernel: failed to load {}: error: {}\n", cstr(init_p), -err);
        panic!("could not load init process");
    }

    arch_proc_init(init);

    let cmdline = (*(*boot).modules).cmdline as *const u8;
    let argp    = [ cmdline, core::ptr::null() ];
    let envp    = [ core::ptr::null() ];

    arch_init_execve(init, 2, &argp as *const *const u8, 1, &envp as *const *const u8);

    /*
#if EARLYCON_DISABLE_ON_INIT
    earlycon_disable();
#endif
    */

    sched_init_spawn(init);
    panic!("scheduler failed to spawn init");
}

