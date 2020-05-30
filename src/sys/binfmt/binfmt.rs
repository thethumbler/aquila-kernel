use prelude::*;
use fs::*;

use sys::process::*;
use sys::binfmt::elf::*;
use sys::thread::*;
use mm::*;

/** binary format */
pub struct BinaryFormat {
    pub check: Option<unsafe fn(vnode: *mut Vnode) -> isize>,
    pub load:  Option<unsafe fn(proc: *mut Process, path: *const u8, vnode: *mut Vnode) -> isize>,
}

/* XXX */
pub const USER_STACK      : usize = 0xC0000000;
pub const USER_STACK_SIZE : usize = 8192 * 1024;
pub const USER_STACK_BASE : usize = USER_STACK - USER_STACK_SIZE;

const NR_BINFMT: usize = 1;

static BINFMT_LIST: [BinaryFormat; NR_BINFMT] = [
    BinaryFormat { check: Some(binfmt_elf_check), load: Some(binfmt_elf_load) },
];

unsafe fn binfmt_fmt_load(proc: *mut Process, path: *const u8, vnode: *mut Vnode, binfmt: *const BinaryFormat, proc_ref: *mut *mut Process) -> isize {
    let mut err = 0;

    (*proc).vm_space.destroy();

    err = (*binfmt).load.unwrap()(proc, path, vnode);
    if err != 0 {
        return err;
    }

    kfree((*proc).name);
    (*proc).name = strdup(path);

    /* Align heap */
    (*proc).heap_start = page_round!((*proc).heap_start);
    (*proc).heap = (*proc).heap_start;

    /* Create heap vm_entry */
    let heap_vm = Box::leak(VmEntry::alloc());

    heap_vm.base  = (*proc).heap_start;
    heap_vm.size  = 0;
    heap_vm.flags = VM_URW;
    heap_vm.qnode = (*proc).vm_space.vm_entries.enqueue(heap_vm);

    heap_vm.vm_object = core::ptr::null_mut();
    
    (*proc).heap_vm  = heap_vm;

    /* Create stack vm_entry */
    let stack_vm = Box::leak(VmEntry::alloc());

    stack_vm.base  = USER_STACK_BASE;
    stack_vm.size  = USER_STACK_SIZE;
    stack_vm.flags = VM_URW;
    stack_vm.qnode = (*proc).vm_space.vm_entries.enqueue(stack_vm);

    stack_vm.vm_object = core::ptr::null_mut();

    (*proc).stack_vm  = stack_vm;

    let thread = (*(*proc).threads.head).value as *mut Thread;
    (*thread).stack.base = USER_STACK_BASE;
    (*thread).stack.size = USER_STACK_BASE;

    return 0;
}

pub unsafe fn binfmt_load(proc: *mut Process, path: *const u8, proc_ref: *mut *mut Process) -> isize {
    let mut err = 0;

    let mut uio: UserOp = core::mem::zeroed();
    //memset(&uio, 0, core::mem::size_of::(struct uio));

    if !proc.is_null() {
        uio = proc_uio!(proc);
    }

    let mut vnode = core::ptr::null_mut();
    err = vfs_lookup(path, &mut uio, &mut vnode, core::ptr::null_mut());
    if err != 0 {
        return err;
    }

    for i in 0..NR_BINFMT {
        if BINFMT_LIST[i].check.unwrap()(vnode) == 0 {
            binfmt_fmt_load(proc, path, vnode, &BINFMT_LIST[i], proc_ref);
            //vfs_close(vnode);
            return 0;
        }
    }

    //vfs_close(vnode);
    return -ENOEXEC;
}
