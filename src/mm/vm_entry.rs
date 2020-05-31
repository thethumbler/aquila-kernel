use prelude::*;

use mm::*;

malloc_define!(M_VM_ENTRY, "vm-entry\0", "virtual memory map entry\0");

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

    pub fn new() -> Self {
        VmEntry::none()
    }
    
    pub fn alloc(val: VmEntry) -> Box<VmEntry> {
        Box::new_tagged(&M_VM_ENTRY, val)
    }

    /** destroy all resources associated with a vm entry */
    pub fn destroy(&mut self) {
        unsafe {
            if !self.vm_anon.is_null() {
                (*self.vm_anon).decref();
            }

            if !self.vm_object.is_null() {
                (*self.vm_object).decref();
            }
        }
    }
}
