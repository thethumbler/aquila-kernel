use prelude::*;

use mm::vmm::kvm_space;
use mm::mm::*;
use crate::include::core::types::*;
use crate::include::mm::mm::*;
use crate::include::mm::vm::*;
use crate::include::mm::kvmem::*;

use crate::mm::vm_object::*;
use crate::mm::vm_space::*;
use crate::fs::*;
use crate::fs::read::*;

use crate::{page_align};

//unsafe fn vm_page_eq(a: *mut u8, b: *mut u8) -> isize {
//    let a = a as *mut VmPage;
//    let b = b as *mut size_t;
//
//    return ((*a).off == *b) as isize;
//}

/** create a new `vm_object` associated with a `vnode` */
pub unsafe fn vm_object_vnode(vnode: *mut Vnode) -> *mut VmObject {
    if vnode.is_null() {
        return core::ptr::null_mut();
    }

    if (*vnode).vm_object.is_null() {
        //struct vm_object *vm_object;

        let vm_object = kmalloc(core::mem::size_of::<VmObject>(), &M_VM_OBJECT, M_ZERO) as *mut VmObject;
        if vm_object.is_null() {
            return core::ptr::null_mut();
        }

        (*vm_object).objtype = VMOBJ_FILE as isize;
        (*vm_object).pages = HashMap::alloc();

        if (*vm_object).pages.is_null() {
            kfree(vm_object as *mut u8);
            return core::ptr::null_mut();
        }

        (*vm_object).pager = &mut vnode_pager;
        (*vm_object).p = vnode as *mut u8;

        (*vnode).vm_object = vm_object;
    }

    return (*vnode).vm_object;
}


/* XXX */
#[repr(C, align(4096))]
struct L {
    page: [u8; PAGE_SIZE],
}

static __load: L = L { page: [0; PAGE_SIZE] };

pub unsafe fn vnode_page_in(vm_object: *mut VmObject, off: off_t) -> *mut VmPage {
    let vm_page = mm_page_alloc();
    if vm_page.is_null() {
        return core::ptr::null_mut();
    }

    (*vm_page).vm_object = vm_object;
    (*vm_page).off = page_align!(off) as off_t;
    (*vm_page).refcnt = 1;

    let vnode = (*vm_object).p as *mut Vnode;

    mm_page_map(kvm_space.pmap, &__load as *const _ as usize, (*vm_page).paddr, VM_KW as isize);
    vfs_read(vnode, (*vm_page).off, PAGE_SIZE, &__load.page as *const _ as *mut u8);

    vm_object_page_insert(vm_object, vm_page);

    return vm_page;
}

unsafe fn vnode_page_out(vm_object: *mut VmObject, off: off_t) -> isize {
    /* no-op */
    return 0;
}

pub static mut vnode_pager: VmPager = VmPager {
    page_in:  vnode_page_in,
    page_out: vnode_page_out,
};
