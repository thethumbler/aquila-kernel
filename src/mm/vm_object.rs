use prelude::*;

use crate::include::core::types::*;
use crate::include::mm::vm::*;
use crate::include::mm::kvmem::*;
use crate::malloc_define;

malloc_define!(M_VM_OBJECT, "vm-object\0", "virtual memory object\0");

/**
 * cached object
 */
#[repr(C)]
pub struct VmObject {
    /** `vm_page`s loaded/contained in the vm object */
    pub pages: *mut HashMap<off_t, *mut VmPage>,

    /** type of the object */
    pub objtype: isize,

    /** number of vm entries referencing this object */
    pub refcnt: usize,

    /** pager for the vm object */
    pub pager: *mut VmPager,

    /** pager private data */
    pub p: *mut u8,
}

pub unsafe fn vm_object_incref(vm_object: *mut VmObject) {
    (*vm_object).refcnt += 1;
}

pub unsafe fn vm_object_decref(vm_object: *mut VmObject) {
    (*vm_object).refcnt -= 1;
    return;

    /*
    if (vm_object->ref == 0) {
        HashMap::free(vm_object->pages);
        kfree(vm_object);
    }
    */
}

pub unsafe fn vm_object_page_insert(vm_object: *mut VmObject, vm_page: *mut VmPage) {
    let pages = (*vm_object).pages;
    (*pages).insert(&((*vm_page).off as off_t), vm_page);
}
