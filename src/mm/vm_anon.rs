use prelude::*;

use mm::mm::*;
use crate::include::core::types::*;
use crate::include::bits::errno::*;

use crate::malloc_define;
use crate::include::mm::kvmem::*;
use crate::include::mm::mm::*;
use crate::include::mm::vm::*;
use crate::alloc::*;

malloc_define!(M_VM_ANON, "vm-anon\0", "anonymous virtual memory object\0");

/** 
 * anonymous memory object
 */
#[repr(C)]
pub struct VmAnon {
    /** hashmap of `VmAref` structures loaded/contained in this anon */
    pub arefs: *mut HashMap<off_t, *mut VmAref>,

    /** number of [VmEntry] structures referencing this anon */
    pub refcnt: usize,

    /** flags associated with the anon */
    pub flags: usize,
}

impl VmAnon {
    /** create new anon structure */
    pub fn new() -> *mut VmAnon {
        unsafe {
            let vm_anon = kmalloc(core::mem::size_of::<VmAnon>(), &M_VM_ANON, M_ZERO) as *mut VmAnon;

            if vm_anon.is_null() {
                return core::ptr::null_mut();
            }

            (*vm_anon).arefs = HashMap::alloc();

            if (*vm_anon).arefs.is_null() {
                kfree(vm_anon as *mut u8);
                return core::ptr::null_mut();
            }

            return vm_anon;
        }
    }

    /** increment number of references to a vm anon */
    pub fn incref(&mut self) -> () {
        self.refcnt += 1;
    }

    /** decrement number of references to a vm anon and destroy it when it reaches zero */
    pub fn decref(&mut self) -> () {
        self.refcnt -= 1;

        if self.refcnt == 0 {
            unsafe {
                vm_anon_destroy(self);
                kfree(self as *const _ as *mut u8);
            }
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
                (*aref).refcnt += 1;
            }

            return 0;
        }
    }

    /** clone an existing anon into a new anon */
    pub fn copy(&self) -> *mut VmAnon {
        unsafe {
            let new_anon = vm_anon_new();

            if new_anon.is_null() {
                return core::ptr::null_mut();
            }

            (*new_anon).flags = self.flags & !VM_COPY;
            (*new_anon).refcnt = 1;

            /* copy all arefs */
            if self.copy_arefs(new_anon) != 0 {
                kfree(new_anon as *mut u8);
                return core::ptr::null_mut();
            }

            return new_anon;
        }
    }

}

pub unsafe fn vm_anon_new() -> *mut VmAnon {
    let vm_anon = kmalloc(core::mem::size_of::<VmAnon>(), &M_VM_ANON, M_ZERO) as *mut VmAnon;

    if vm_anon.is_null() {
        return core::ptr::null_mut();
    }

    (*vm_anon).arefs = HashMap::alloc();

    if (*vm_anon).arefs.is_null() {
        kfree(vm_anon as *mut u8);
        return core::ptr::null_mut();
    }

    return vm_anon;
}

/** destroy all resources associated with an aref */
pub unsafe fn vm_aref_destroy(_vm_aref: *mut VmAref) -> () {
    /* nothing to do */
}

/** decrement references to an aref */
pub unsafe fn vm_aref_decref(vm_aref: *mut VmAref) -> () {
    (*vm_aref).refcnt -= 1;
}

/** destroy all resources associated with an anon */
pub unsafe fn vm_anon_destroy(vm_anon: *mut VmAnon) -> () {
    if vm_anon.is_null() {
        return;
    }

    let arefs = (*vm_anon).arefs;

    if arefs.is_null() {
        return;
    }

    for node in (*arefs).iter() {
        let aref = node.value;

        vm_aref_decref(aref);

        if (*aref).refcnt == 0 {
            if !(*aref).vm_page.is_null() {
                mm_page_dealloc((*(*aref).vm_page).paddr);
            }

            kfree(aref as *mut u8);
        }
    }

    let arefs = (*vm_anon).arefs.replace(HashMap::empty());
    arefs.free();

    let arefs_ptr = (*vm_anon).arefs;
    (*vm_anon).arefs = core::ptr::null_mut();
    kfree(arefs_ptr as *mut u8);
}

/** increment number of references to a vm anon */
pub unsafe fn vm_anon_incref(vm_anon: *mut VmAnon) -> () {
    if vm_anon.is_null() {
        return;
    }

    (*vm_anon).refcnt += 1;
}

/** decrement number of references to a vm anon and destroy it when it reaches zero */
pub unsafe fn vm_anon_decref(vm_anon: *mut VmAnon) -> () {
    if vm_anon.is_null() {
        return;
    }

    (*vm_anon).refcnt -= 1;

    if (*vm_anon).refcnt == 0 {
        vm_anon_destroy(vm_anon);
        kfree(vm_anon as *mut u8);
    }
}

/** copy all aref structures from ```src``` to ```dst``` */
pub unsafe fn vm_anon_copy_arefs(src: *mut VmAnon, dst: *mut VmAnon) -> isize {
    if src.is_null() || dst.is_null() || (*src).arefs.is_null() || (*dst).arefs.is_null() {
        return -EINVAL;
    }

    let s_arefs = &mut *(*src).arefs;
    let d_arefs = &mut *(*dst).arefs;

    /* copy all arefs */
    for node in s_arefs.iter() {
        let aref = node.value;

        d_arefs.insert(&node.key, aref);
        (*aref).refcnt += 1;
    }

    return 0;
}

/** clone an existing anon into a new anon */
pub unsafe fn vm_anon_copy(vm_anon: *mut VmAnon) -> *mut VmAnon {
    if vm_anon.is_null() {
        return core::ptr::null_mut();
    }

    let new_anon = vm_anon_new();

    if new_anon.is_null() {
        return core::ptr::null_mut();
    }

    (*new_anon).flags = (*vm_anon).flags & !VM_COPY;
    (*new_anon).refcnt = 1;

    /* copy all arefs */
    if vm_anon_copy_arefs(vm_anon, new_anon) != 0 {
        kfree(new_anon as *mut u8);
        return core::ptr::null_mut();
    }

    return new_anon;
}

/**
 * anonymous memory object reference
 *
 * The [VmAref] acts as a container of a [VmPage] with reference count
 * of anonymous memory regions ([VmAnon]) referencing/using that page.
 */
#[repr(C)]
pub struct VmAref {
    /** vm page associated with the aref */
    pub vm_page: *mut VmPage,

    /** number of references to the aref */
    pub refcnt: usize,

    /** flags associated with this aref */
    pub flags: usize,
}
