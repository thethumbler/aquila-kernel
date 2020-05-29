use prelude::*;
use mm::*;

malloc_define!(M_ANON_REF, "anon-ref\0", "anonymous virtual memory object reference\0");

pub struct VmAref {
    /** vm page associated with the aref */
    pub vm_page: *mut VmPage,

    /** number of references to the aref */
    refcnt: usize,

    /** flags associated with this aref */
    pub flags: usize,
}

impl VmAref {
    pub fn alloc() -> Box<VmAref> {
        unsafe { Box::new_zeroed_tagged(&M_ANON_REF).assume_init() }
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
