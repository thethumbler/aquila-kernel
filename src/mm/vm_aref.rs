use prelude::*;
use mm::*;

malloc_define!(M_VM_AREF, "vm-aref\0", "anonymous virtual memory object reference\0");

pub struct VmAref {
    /** vm page associated with the aref */
    pub vm_page: *mut VmPage,

    /** number of references to the aref */
    refcnt: usize,

    /** flags associated with this aref */
    pub flags: usize,
}

impl VmAref {
    pub fn new() -> VmAref {
        VmAref {
            vm_page: core::ptr::null_mut(),
            refcnt: 0,
            flags: 0,
        }
    }

    pub fn alloc(val: VmAref) -> Box<VmAref> {
        Box::new_tagged(&M_VM_AREF, val)
    }

    /** destroy all resources associated with an aref */
    pub fn destroy(&mut self) {
        /* nothing to do */
    }

    /** get references count to an aref */
    pub fn refcnt(&self) -> usize {
        self.refcnt
    }

    /** increment references to an aref */
    pub fn incref(&mut self) {
        self.refcnt += 1;
    }

    /** decrement references to an aref */
    pub fn decref(&mut self) {
        self.refcnt -= 1;
    }
}
