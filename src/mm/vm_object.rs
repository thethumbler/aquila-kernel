use prelude::*;

use mm::vmm::*;
use mm::kvmem::*;

malloc_define!(M_VM_OBJECT, "vm-object\0", "virtual memory object\0");

/**
 * cached object
 */
pub struct VmObject {
    /** `vm_page`s loaded/contained in the vm object */
    pub pages: *mut HashMap<off_t, *mut VmPage>,

    /** type of the object */
    pub objtype: isize,

    /** number of vm entries referencing this object */
    refcnt: usize,

    /** pager for the vm object */
    pub pager: *mut VmPager,

    /** pager private data */
    pub p: *mut u8,
}

impl VmObject {
    pub fn incref(&mut self) {
        self.refcnt += 1;
    }

    pub fn decref(&mut self) {
        self.refcnt -= 1;
    }

    pub fn refcnt(&self) -> usize {
        self.refcnt
    }

    pub fn insert(&mut self, vm_page: *mut VmPage) {
        unsafe {
            let pages = self.pages;
            (*pages).insert(&((*vm_page).off as off_t), vm_page);
        }
    }
}
