use prelude::*;

use mm::mm::*;
use crate::mm::*;
use crate::include::mm::vm::*;
use crate::include::mm::mm::*;
use crate::include::mm::kvmem::*;

use crate::include::core::string::*;
use crate::malloc_define;

malloc_define!(M_VM_AREF, "vm-aref\0", "anonymous virtual memory object reference\0");

#[no_mangle]
pub static mut kvm_space: VmSpace = VmSpace {
    pmap: core::ptr::null_mut() as *mut PhysicalMap,
    vm_entries: Queue::empty(),
};

pub unsafe fn vm_map(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> isize {
    //printk(b"vm_map(vm_space=%p, vm_entry=%p)\n\0".as_ptr(), vm_space, vm_entry);
    return mm_map((*vm_space).pmap, (*vm_entry).paddr, (*vm_entry).base, (*vm_entry).size, (*vm_entry).flags as isize);
}

pub unsafe fn vm_unmap(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> () {
    //printk(b"vm_unmap(vm_space=%p, vm_entry=%p)\n\0".as_ptr(), vm_space, vm_entry);

    if ((*vm_entry).flags & VM_SHARED) != 0 {
        /* TODO */
    } else {
        mm_unmap((*vm_space).pmap, (*vm_entry).base, (*vm_entry).size);
    }
}

#[no_mangle]
pub unsafe fn vm_unmap_full(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> () {
    //printk(b"vm_unmap(vm_space=%p, vm_entry=%p)\n\0".as_ptr(), vm_space, vm_entry);

    if ((*vm_entry).flags & VM_SHARED) != 0 {
        /* TODO */
    } else {
        mm_unmap_full((*vm_space).pmap, (*vm_entry).base, (*vm_entry).size);
    }
}
