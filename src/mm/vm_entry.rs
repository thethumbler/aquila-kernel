use prelude::*;

use crate::mm::*;

use crate::include::mm::vm::*;
use crate::include::mm::kvmem::*;
use crate::malloc_define;

malloc_define!(M_VM_ENTRY, "vm_entry\0", "virtual memory map entry\0");

#[derive(Copy, Clone, Debug)]
pub struct VmEntry {
    /** physical Address, 0 means anywhere - XXX */
    pub paddr: paddr_t,

    /** address of the vm entry inside the `vm_space` */
    pub base: usize, //vaddr_t,

    /** size of the vm entry */
    pub size: usize,

    /** permissions flags */
    pub flags: usize,

    /** anon layer object */
    pub vm_anon: *mut VmAnon,

    /** backening object */
    pub vm_object: *mut VmObject,

    /** offset inside object */
    pub off: usize,

    /** the queue node this vm entry is stored in */
    pub qnode: *mut QueueNode<*mut VmEntry>
}

impl VmEntry {
    pub const fn none() -> Self {
        VmEntry {
            paddr: 0,
            base: 0,
            size: 0,
            flags: 0,
            vm_anon: core::ptr::null_mut(),
            vm_object: core::ptr::null_mut(),
            off: 0,
            qnode: core::ptr::null_mut(),
        }
    }
}

/** create new vm entry */
pub unsafe fn vm_entry_new() -> *mut VmEntry {
    let vm_entry = kmalloc(core::mem::size_of::<VmEntry>(), &M_VM_ENTRY, M_ZERO) as *mut VmEntry;

    if vm_entry.is_null() {
        return core::ptr::null_mut();
    }

    return vm_entry;
}

/** destroy all resources associated with a vm entry */
pub unsafe fn vm_entry_destroy(vm_entry: *mut VmEntry) {
    if vm_entry.is_null() {
        return;
    }

    if !(*vm_entry).vm_anon.is_null() {
        vm_anon_decref((*vm_entry).vm_anon);
    }

    if !(*vm_entry).vm_object.is_null() {
        vm_object_decref((*vm_entry).vm_object);
    }
}
