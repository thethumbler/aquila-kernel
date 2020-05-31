use prelude::*;
use mm::*;

malloc_define!(M_VM_ANON, "vm-anon\0", "anonymous virtual memory object\0");

/** 
 * anonymous memory object
 */
pub struct VmAnon {
    /** hashmap of `VmAref` structures loaded/contained in this anon */
    pub arefs: *mut HashMap<off_t, *mut VmAref>,

    /** number of [VmEntry] structures referencing this anon */
    refcnt: usize,

    /** flags associated with the anon */
    pub flags: usize,
}

impl VmAnon {
    pub const fn empty() -> VmAnon {
        VmAnon {
            arefs: core::ptr::null_mut(),
            refcnt: 0,
            flags: 0,
        }
    }

    pub fn new() -> VmAnon {
        VmAnon {
            arefs: HashMap::alloc(),

            ..VmAnon::empty()
        }
    }

    /** create new anon structure */
    pub fn alloc(val: VmAnon) -> Box<VmAnon> {
        Box::new_tagged(&M_VM_ANON, val)
    }

    /** increment number of references to a vm anon */
    pub fn incref(&mut self) {
        self.refcnt += 1;
    }

    /** decrement number of references to a vm anon and destroy it when it reaches zero */
    pub fn decref(&mut self) {
        self.refcnt -= 1;

        if self.refcnt == 0 {
            unsafe {
                self.destroy();

                // XXX
                Box::from_raw(self);
            }
        }
    }

    pub fn refcnt(&self) -> usize {
        self.refcnt
    }

    /** destroy all resources associated with an anon */
    pub fn destroy(&mut self) {
        unsafe {
            let arefs = self.arefs;

            if arefs.is_null() {
                return;
            }

            for node in (*arefs).iter() {
                let aref = &mut *node.value;

                aref.decref();

                if aref.refcnt() == 0 {
                    if !aref.vm_page.is_null() {
                        mm_page_dealloc((*aref.vm_page).paddr);
                    }

                    Box::from_raw(aref);
                }
            }

            let arefs = self.arefs.replace(HashMap::empty());
            arefs.free();

            let arefs_ptr = self.arefs;
            self.arefs = core::ptr::null_mut();

            Box::from_raw(arefs_ptr);
        }
    }

    /** copy all aref structures from ```src``` to ```dst``` */
    fn copy_arefs(&self, dst: *mut VmAnon) -> isize {
        unsafe {
            if dst.is_null() || self.arefs.is_null() || (*dst).arefs.is_null() {
                return -EINVAL;
            }

            let s_arefs = &mut *self.arefs;
            let d_arefs = &mut *(*dst).arefs;

            /* copy all arefs */
            for node in s_arefs.iter() {
                let aref = node.value;

                d_arefs.insert(&node.key, aref);
                (*aref).incref();
            }

            return 0;
        }
    }

    /** clone an existing anon into a new anon */
    pub fn copy(&self) -> *mut VmAnon {
        unsafe {
            let new_anon = Box::leak(VmAnon::alloc(VmAnon::new()));

            new_anon.flags = self.flags & !VM_COPY;
            new_anon.refcnt = 1;

            /* copy all arefs */
            if self.copy_arefs(new_anon) != 0 {
                Box::from_raw(new_anon);
                return core::ptr::null_mut();
            }

            return new_anon;
        }
    }

}
