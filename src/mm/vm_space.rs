use prelude::*;

use arch::i386::mm::i386::*;
use crate::mm::*;

use crate::include::mm::vm::*;
use crate::include::mm::mm::*;
use crate::include::mm::kvmem::*;
use crate::include::mm::pmap::*;

use crate::{page_align, malloc_declare, print};

malloc_declare!(M_VM_ENTRY);

/** 
 * virtual memory space
 *
 * This structure holds the entire mapping of a virtual memory 
 * space/view
 */
#[repr(C)]
#[derive(Debug)]
pub struct VmSpace {
    /** physical memory mapper (arch-specific) */
    pub pmap: *mut PhysicalMap,

    /** virtual memory regions inside the vm space */
    pub vm_entries: Queue<VmEntry>,
}

unsafe impl Sync for VmSpace {}

impl VmSpace {
    /** look for the [VmEntry] containing ```vaddr``` inside the [VmSpace] */
    pub fn find(&self, vaddr: usize) -> Option<&VmEntry> {
        let vaddr = page_align!(vaddr);

        let vm_entries = &self.vm_entries;

        for qnode in vm_entries.iter() {
            let vm_entry = unsafe { &*(qnode.value as *mut VmEntry) };

            let vm_end = vm_entry.base + vm_entry.size;
            if vaddr >= vm_entry.base && vaddr < vm_end {
                return Some(vm_entry);
            }
        }

        None
    }

    pub fn insert(&mut self, vm_entry: &mut VmEntry) -> isize {
        let queue = &mut self.vm_entries;
        let alloc = vm_entry.base == 0;

        let mut end = vm_entry.base + vm_entry.size;

        let mut cur = core::ptr::null_mut() as *mut QueueNode<VmEntry>;
        let mut prev_end = 0usize;

        if alloc {
            /* look for the last valid entry */
            for qnode in queue.iter() {
                let cur_vm_entry = unsafe { &*(qnode.value as *mut VmEntry) };

                if cur_vm_entry.base - prev_end >= vm_entry.size {
                    vm_entry.base = cur_vm_entry.base - vm_entry.size;
                    end = vm_entry.base + vm_entry.size;
                }

                prev_end = cur_vm_entry.base + cur_vm_entry.size;
            }
        }

        //if (!prev_end) {
        //    vm_entry->base = (uintptr_t)-1 - vm_entry->size;
        //}

        for qnode in queue.iter() {
            let cur_vm_entry = unsafe { &*(qnode.value as *mut VmEntry) };

            if vm_entry.base != 0 && cur_vm_entry.base >= end && prev_end <= vm_entry.base {
                cur = qnode as *const _ as *mut QueueNode<VmEntry>;
                break;
            }

            prev_end = cur_vm_entry.base + cur_vm_entry.size;
        }

        if cur.is_null() {
            //return -ENOMEM;
            return -1;
        }

        unsafe {
            let node = kmalloc(core::mem::size_of::<QueueNode<VmEntry>>(), &M_QNODE, M_ZERO) as *mut QueueNode<VmEntry>;

            (*node).value = vm_entry;
            (*node).next  = cur;
            (*node).prev  = (*cur).prev;

            if !(*cur).prev.is_null() {
                (*(*cur).prev).next = &*node as *const _ as *mut QueueNode<VmEntry>;
            }

            (*cur).prev = &*node as *const _ as *mut QueueNode<VmEntry>;
            vm_entry.qnode = &*node as *const _ as *mut QueueNode<VmEntry>;
            (*queue).count += 1;

            return 0;
        }
    }

    pub fn fork(&mut self, dst: &mut VmSpace) -> isize {
        /* copy vm entries */
        let src_vm_entries = &mut self.vm_entries;

        for qnode in src_vm_entries.iter() {
            unsafe {
                let s_entry = unsafe { &*(qnode.value as *mut VmEntry) };
                let mut d_entry = unsafe { kmalloc(core::mem::size_of::<VmEntry>(), &M_VM_ENTRY, M_ZERO) as *mut VmEntry };

                *d_entry = *s_entry;
                (*d_entry).qnode = (*dst).vm_entries.enqueue(&mut *d_entry);

                if !s_entry.vm_anon.is_null() {
                    let vm_anon = unsafe { &mut *s_entry.vm_anon };

                    vm_anon.flags |= VM_COPY;
                    vm_anon.incref();
                }

                if !s_entry.vm_object.is_null() {
                    let vm_object = unsafe { &mut *s_entry.vm_object };
                    vm_object.refcnt += 1;
                }

                if (s_entry.flags & (VM_UW|VM_KW)) != 0 && (s_entry.flags & VM_SHARED) == 0 {
                    /* remove write permission from all pages */
                    let sva = s_entry.base;
                    let eva = sva + s_entry.size;
                    let flags = s_entry.flags & !(VM_UW|VM_KW);

                    unsafe { pmap_protect(self.pmap, sva, eva, flags as u32); }
                }
            }
        }

        return 0;
    }
}

pub unsafe fn vm_space_insert(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> isize {
    if vm_space.is_null() || vm_entry.is_null() {
        //return -EINVAL;
        return -1;
    }

    (*vm_space).insert(&mut *vm_entry)

    /*
    let queue = &mut (*vm_space).vm_entries;

    let alloc = (*vm_entry).base == 0;

    let mut end = (*vm_entry).base as usize + (*vm_entry).size;

    let mut cur = core::ptr::null_mut() as *mut Qnode;
    let mut prev_end = 0usize;

    if alloc {
        /* look for the last valid entry */
        let mut qnode = queue.head;
        while !qnode.is_null() {
            let cur_vm_entry = (*qnode).value as *mut VmEntry;

            if (*cur_vm_entry).base as usize - prev_end >= (*vm_entry).size as usize {
                (*vm_entry).base = (*cur_vm_entry).base - (*vm_entry).size;
                end = ((*vm_entry).base + (*vm_entry).size) as usize;
            }

            prev_end = ((*cur_vm_entry).base + (*cur_vm_entry).size) as usize;

            qnode = (*qnode).next;
        }
    }

    //if (!prev_end) {
    //    vm_entry->base = (uintptr_t)-1 - vm_entry->size;
    //}

    for qnode in (*queue).iter() {
        let cur_vm_entry = (*qnode).value as *mut VmEntry;

        if (*vm_entry).base != 0 && (*cur_vm_entry).base >= end && prev_end <= (*vm_entry).base as usize {
            cur = qnode as *const _ as *mut Qnode;
            break;
        }

        prev_end = ((*cur_vm_entry).base + (*cur_vm_entry).size) as usize;

        //qnode = (*qnode).next;
    }

    if cur.is_null() {
        //return -ENOMEM;
        return -1;
    }

    let node = kmalloc(core::mem::size_of::<Qnode>(), &M_QNODE, M_ZERO) as *mut Qnode;
    if node.is_null() {
        /* TODO */
        return -1;
    }

    (*node).value = vm_entry as *mut u8;
    (*node).next  = cur;
    (*node).prev  = (*cur).prev;

    if !(*cur).prev.is_null() {
        (*(*cur).prev).next = node;
    }

    (*cur).prev  = node;
    (*vm_entry).qnode = node;
    (*queue).count += 1;

    return 0;
    */
}

/*
 * \ingroup mm
 * \brief lookup the vm entry containing `vaddr` inside a vm space
 */
pub unsafe fn vm_space_find(vm_space: *mut VmSpace, vaddr: usize) -> *mut VmEntry {
    if vm_space.is_null() {
        return core::ptr::null_mut();
    }

    /*
    let vaddr = page_align!(vaddr);

    let vm_entries = &mut (*vm_space).vm_entries;

    for qnode in (*vm_entries).iter() {
        let vm_entry = (*qnode).value as *mut VmEntry;
        let vm_end = (*vm_entry).base + (*vm_entry).size;

        if vaddr >= (*vm_entry).base && vaddr < vm_end {
            return vm_entry;
        }
    }
    */

    let r = (*vm_space).find(vaddr);

    if r.is_some() {
        r.unwrap() as *const _ as *mut VmEntry
    } else {
        core::ptr::null_mut()
    }

    //return core::ptr::null_mut();
}

/*
 * \ingroup mm
 * \brief destroy all resources associated with a vm space
 */
pub unsafe fn vm_space_destroy(vm_space: *mut VmSpace) -> () {
    if vm_space.is_null() {
        return;
    }

    let vm_entries = &mut (*vm_space).vm_entries;
    let mut vm_entry = (*vm_entries).dequeue();

    while !vm_entry.is_null() {
        vm_entry_destroy(vm_entry);
        kfree(vm_entry as *mut u8);
        vm_entry = (*vm_entries).dequeue();
    }

    pmap_remove_all((*vm_space).pmap);
}

/*
 * \ingroup mm
 * \brief fork a vm space into another vm space
 */
pub unsafe fn vm_space_fork(src: *mut VmSpace, dst: *mut VmSpace) -> isize {
    if src.is_null() || dst.is_null() {
        //return -EINVAL;
        return -1;
    }

    (*src).fork(&mut *dst)

    /*

    /* copy vm entries */
    let src_vm_entries = &mut (*src).vm_entries;

    let mut qnode = (*src_vm_entries).head;

    while !qnode.is_null() {
        let s_entry = (*qnode).value as *mut VmEntry;
        let d_entry = kmalloc(core::mem::size_of::<VmEntry>(), &M_VM_ENTRY, 0) as *mut VmEntry;

        if d_entry.is_null() {
            /* TODO */
        }

        //memcpy(d_entry, s_entry, sizeof(struct vm_entry));
        *d_entry = *s_entry;

        (*d_entry).qnode = enqueue(&mut (*dst).vm_entries, d_entry as *mut u8);

        if !(*s_entry).vm_anon.is_null() {
            (*(*s_entry).vm_anon).flags |= VM_COPY;
            vm_anon_incref((*s_entry).vm_anon);
            //s_entry->vm_anon->ref++;
        }

        if !(*s_entry).vm_object.is_null() {
            (*(*s_entry).vm_object).refcnt += 1;
        }

        if ((*s_entry).flags & (VM_UW|VM_KW)) != 0 && ((*s_entry).flags & VM_SHARED) == 0 {
            /* remove write permission from all pages */
            let sva = (*s_entry).base;
            let eva = sva + (*s_entry).size;
            let flags = (*s_entry).flags & !(VM_UW|VM_KW);

            pmap_protect((*src).pmap, sva, eva, flags as u32);
        }

        qnode = (*qnode).next;
    }

    return 0;
    */
}
