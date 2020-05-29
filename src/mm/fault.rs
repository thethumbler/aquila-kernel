use prelude::*;

use arch::mm::i386::*;
use mm::*;
use sys::sched::*;
use sys::signal::*;

/** a structure holding parameters relevant to a page fault */
struct FaultInfo {
    flags: usize,
    addr: usize,

    vm_space: *mut AddressSpace,
    vm_entry: *mut VmEntry,

    off: off_t,
}

fn check_violation(flags: usize, vm_flags: usize) -> isize {
    /* returns 1 on violation, 0 otherwise */
    return (
    ((flags & PF_READ)  != 0 && (vm_flags & VM_UR) == 0) ||
    ((flags & PF_WRITE) != 0 && (vm_flags & VM_UW) == 0) ||
    ((flags & PF_EXEC)  != 0 && (vm_flags & VM_UX) == 0)) as isize;
}

/**
 * handle the page fault if the page is already present
 * in virtual memory mapping
 */
unsafe fn pf_present(info: *mut FaultInfo) -> isize {
    /* page is already present but with incorrect permissions */
    let vm_entry = (*info).vm_entry;
    let vm_anon  = (*vm_entry).vm_anon;

    let pmap = (*(*info).vm_space).pmap;

    /* if there is no anon or the anon is shared
     * we can't handle it here and have to fallthrough to 
     * other handlers
     */
    if vm_anon.is_null() || (*vm_anon).refcnt != 1 {
        return 0;
    }

    /* we own the anon */
    //let off = (*info).off as u32;
    let hash_node = (*(*vm_anon).arefs).lookup(&(*info).off);
    let vm_aref = if hash_node.is_some() { hash_node.unwrap().value } else { core::ptr::null_mut() };

    if vm_aref.is_null() || (*vm_aref).refcnt() != 1 {
        return 0;
    }

    if ((*vm_aref).flags & VM_COPY) != 0 {
        /* copy page */
        let new_page = mm_page_alloc();
        (*new_page).off = (*(*vm_aref).vm_page).off;
        (*new_page).refcnt = 1;
        (*new_page).vm_object = core::ptr::null_mut();

        pmap_page_copy((*(*vm_aref).vm_page).paddr, (*new_page).paddr);

        mm_page_decref((*(*vm_aref).vm_page).paddr);

        (*vm_aref).vm_page = new_page;
        (*vm_aref).flags &= !VM_COPY;

        mm_page_map(pmap, (*info).addr, (*new_page).paddr, ((*(*info).vm_entry).flags & VM_PERM) as isize);
    } else {
        /* we own the aref, just change permissions */
        pmap_protect(pmap, (*info).addr, (*info).addr + PAGE_SIZE, ((*(*info).vm_entry).flags & VM_PERM) as u32);
    }

    return 1;
}

unsafe fn pf_anon(info: *mut FaultInfo) -> isize {
    let vm_entry = (*info).vm_entry;
    let mut vm_anon  = (*vm_entry).vm_anon;

    let pmap = (*(*info).vm_space).pmap;

    if (*vm_anon).flags & VM_COPY != 0 {

        if ((*vm_anon).refcnt > 1) {
            let new_anon = (*vm_anon).copy();
            vm_anon_decref(vm_anon);
            (*vm_entry).vm_anon = new_anon;
            vm_anon = new_anon;
        }

        (*vm_anon).flags &= !VM_COPY;
    }

    let aref_node = (*(*vm_anon).arefs).lookup(&(*info).off);

    if aref_node.is_none() || (*aref_node.unwrap()).value.is_null() {
        return 0;
    }

    let aref_node = aref_node.unwrap();

    let aref = (*aref_node).value;

    if (*aref).vm_page.is_null() {
        panic!("aref has no page");
        /* TODO */
    }

    if (*info).flags & PF_WRITE == 0 {
        /* map read-only */
        let vm_page = (*aref).vm_page;

        let perm = ((*vm_entry).flags & VM_PERM) & !(VM_UW|VM_KW);
        mm_page_map(pmap, (*info).addr, (*vm_page).paddr, perm as isize);
        mm_page_incref((*vm_page).paddr);

        return 1;
    }

    /* we have PF_WRITE */

    if (*aref).refcnt() == 1 {
        /* we own the aref, just map */

        let vm_page = (*aref).vm_page;

        let perm = (*vm_entry).flags & VM_PERM;
        mm_page_map(pmap, (*info).addr, (*vm_page).paddr, perm as isize);

        mm_page_incref((*vm_page).paddr);

        return 1;
    }

    /* copy, map read-write */
    (*aref).decref();

    let vm_page = (*aref).vm_page;

    let new_page = mm_page_alloc();
    (*new_page).off = (*vm_page).off;
    (*new_page).refcnt = 1;
    (*new_page).vm_object = core::ptr::null_mut();

    pmap_page_copy((*vm_page).paddr, (*new_page).paddr);

    let mut new_aref = Box::leak(AnonRef::alloc());

    new_aref.incref();
    new_aref.flags = (*aref).flags;
    new_aref.vm_page = new_page;

    (*(*(*vm_entry).vm_anon).arefs).node_remove(aref_node);
    (*(*(*vm_entry).vm_anon).arefs).insert(&(*info).off, new_aref);

    mm_page_map(pmap, (*info).addr, (*new_page).paddr, ((*vm_entry).flags & VM_PERM) as isize);

    return 1;
}

unsafe fn vm_object_page(vm_object: *mut VmObject, off: &off_t) -> *mut VmPage {
    let hash_node = (*(*vm_object).pages).lookup(off);
    let mut vm_page = core::ptr::null_mut();

    if hash_node.is_some() {
        let hash_node = hash_node.unwrap();
        /* page was found in the vm object */
        vm_page = (*hash_node).value;
    } else {
        /* page was not found, page in */
        let pager = (*vm_object).pager;
        if (!pager.is_null() && !((*pager).page_in as *const u8).is_null()) {
            vm_page = ((*pager).page_in)(vm_object, *off);
        } else {
            /* what should we do now? */
            panic!("Shit!");
        }
    }

    return vm_page;
}

unsafe fn pf_object(info: *mut FaultInfo) -> isize {
    let vm_entry = (*info).vm_entry;

    let vm_object = (*vm_entry).vm_object;
    let mut vm_page = core::ptr::null_mut();
    let pmap = (*(*info).vm_space).pmap;

    /* look for page inside the object pages hashmap */
    vm_page = vm_object_page(vm_object, &(*info).off);

    if (*vm_entry).flags & VM_UW == 0 {
        /* read only page -- just map */
        mm_page_incref((*vm_page).paddr);
        mm_page_map(pmap, (*info).addr, (*vm_page).paddr, ((*vm_entry).flags & VM_PERM) as isize);
        return 1;
    }

    /* read-write page -- promote */

    /* allocate a new anon if we don't have one */
    if (*vm_entry).vm_anon.is_null() {
        (*vm_entry).vm_anon = VmAnon::new();
        (*(*vm_entry).vm_anon).refcnt = 1;
    }

    let mut vm_aref = Box::leak(AnonRef::alloc());

    vm_aref.vm_page = vm_page;
    vm_aref.incref();

    //if (!(pf->flags & PF_WRITE)) {
    //    /* just mark for copying */
    //    vm_aref->flags |= VM_COPY;
    //    HashMap::insert(vm_entry->vm_anon->arefs, pf->hash, vm_aref);
    //    uint32_t perms = (vm_entry->flags & VM_PERM) & ~(VM_UW|VM_KW);
    //    mm_page_map(pmap, pf->addr, vm_page->paddr, perms);
    //    return 1;
    //}

    /* copy page */
    let new_page = mm_page_alloc();
    (*new_page).off = (*vm_page).off;
    (*new_page).refcnt = 1;
    (*new_page).vm_object = core::ptr::null_mut();

    pmap_page_copy((*vm_page).paddr, (*new_page).paddr);

    (*vm_aref).vm_page = new_page;
    (*(*(*vm_entry).vm_anon).arefs).insert(&(*info).off, vm_aref);

    mm_page_map(pmap, (*info).addr, (*new_page).paddr, ((*vm_entry).flags & VM_PERM) as isize);


    return 1;
}

unsafe fn pf_zero(info: *mut FaultInfo) -> isize {
    let vm_entry = (*info).vm_entry;
    let pmap = (*(*info).vm_space).pmap;

    if (*vm_entry).vm_anon.is_null() {
        (*vm_entry).vm_anon = VmAnon::new();
        (*(*vm_entry).vm_anon).refcnt = 1;
    }

    //let vm_aref = kmalloc(core::mem::size_of::<VmAref>(), &M_VM_AREF, M_ZERO) as *mut VmAref;
    //if vm_aref.is_null() {
    //    /* TODO */
    //}

    let new_page = mm_page_alloc();
    (*new_page).off = (*info).off;
    (*new_page).refcnt = 1;

    let vm_aref = Box::leak(AnonRef::alloc());

    vm_aref.vm_page = new_page;
    vm_aref.incref();

    (*(*(*vm_entry).vm_anon).arefs).insert(&(*info).off, vm_aref);

    mm_page_map(pmap, (*info).addr, (*new_page).paddr, VM_KW as isize); //vm_entry->flags & VM_PERM);
    //memset((void *) info->addr, 0, PAGE_SIZE);
    core::ptr::write_bytes((*info).addr as *mut u8, 0, PAGE_SIZE as usize);
    pmap_protect(pmap, (*info).addr, (*info).addr + PAGE_SIZE, ((*vm_entry).flags & VM_PERM) as u32);

    return 1;
}

pub unsafe fn mm_page_fault(vaddr: usize, flags: isize) -> () {
    let addr = page_align!(vaddr);

    let vm_space = &mut (*curproc!()).vm_space;
    let pmap = (*vm_space).pmap;
    let mut vm_entry = core::ptr::null_mut();

    /* look for vm_entry that contains the page */
    vm_entry = vm_space_find(vm_space, addr);

    /* segfault if there is no entry or the permissions are incorrect */
    if vm_entry.is_null() || check_violation(flags as usize, (*vm_entry).flags) != 0 {
        //print!("will signal process\n");
        signal_proc_send(curproc!(), SIGSEGV);
        return;
    }

    /* get page offset in object */
    let off = addr - (*vm_entry).base + (*vm_entry).off;

    /* construct page fault structure */
    let mut info = FaultInfo {
        flags: flags as usize,
        addr: addr,
        vm_space: vm_space,
        vm_entry: vm_entry,
        off: off as off_t,
    };

    /* try to handle page present case */
    if (flags as usize & PF_PRESENT) != 0 && pf_present(&mut info) != 0 {
        return;
    }

    /* check the anon layer for the page and handle if present */
    if !(*vm_entry).vm_anon.is_null() && pf_anon(&mut info) != 0 {
        return;
    }

    /* check the backening object for the page and handle if present */
    if !(*vm_entry).vm_object.is_null() && pf_object(&mut info) != 0 {
        return;
    }

    /* just zero out the page */
    if pf_zero(&mut info) != 0 {
        return;
    }

    signal_proc_send(curproc!(), SIGSEGV);
    return;
}
