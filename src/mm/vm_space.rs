use prelude::*;

use mm::*;
use arch::i386::mm::i386::*;

use crate::{page_align, malloc_declare, print};

malloc_declare!(M_VM_ENTRY);

#[derive(Debug)]
pub struct AddressSpace {
    /** physical memory mapper (arch-specific) */
    pub pmap: *mut PhysicalMap,

    /** virtual memory regions inside the vm space */
    pub vm_entries: Queue<*mut VmEntry>,
}

unsafe impl Sync for AddressSpace {}

impl AddressSpace {
    /** look for the [VmEntry] containing ```vaddr``` inside the [AddressSpace] */
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

        let mut cur = core::ptr::null_mut() as *mut QueueNode<*mut VmEntry>;
        let mut prev_end = 0usize;

        if alloc {
            /* look for the last valid entry */
            for qnode in queue.iter() {
                let cur_vm_entry = unsafe { &mut *(qnode.value as *mut VmEntry) };

                if cur_vm_entry.base - prev_end >= vm_entry.size {
                    vm_entry.base = cur_vm_entry.base - vm_entry.size;
                    end = vm_entry.base + vm_entry.size;
                }

                prev_end = cur_vm_entry.base + cur_vm_entry.size;
            }
        }

        for qnode in queue.iter() {
            let cur_vm_entry = unsafe { &*qnode.value };

            if vm_entry.base != 0 && cur_vm_entry.base >= end && prev_end <= vm_entry.base {
                // XXX
                cur = qnode as *const _ as *mut QueueNode<*mut VmEntry>;
                break;
            }

            prev_end = cur_vm_entry.base + cur_vm_entry.size;
        }

        if cur.is_null() {
            //return -ENOMEM;
            return -1;
        }

        vm_entry.qnode = queue.enqueue_before(cur, vm_entry);
        return 0;
    }

    pub fn fork(&mut self, dst: &mut AddressSpace) -> isize {
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

pub unsafe fn vm_space_find(vm_space: *mut AddressSpace, vaddr: usize) -> *mut VmEntry {
    if vm_space.is_null() {
        return core::ptr::null_mut();
    }

    let r = (*vm_space).find(vaddr);

    if r.is_some() {
        r.unwrap() as *const _ as *mut VmEntry
    } else {
        core::ptr::null_mut()
    }
}

pub unsafe fn vm_space_destroy(vm_space: *mut AddressSpace) -> () {
    if vm_space.is_null() {
        return;
    }

    let vm_entries = &mut (*vm_space).vm_entries;
    let mut vm_entry = (*vm_entries).dequeue();

    while !vm_entry.is_none() {
        vm_entry_destroy(vm_entry.unwrap());
        kfree(vm_entry.unwrap() as *mut u8);
        vm_entry = (*vm_entries).dequeue();
    }

    pmap_remove_all((*vm_space).pmap);
}

pub unsafe fn vm_space_fork(src: *mut AddressSpace, dst: *mut AddressSpace) -> isize {
    if src.is_null() || dst.is_null() {
        //return -EINVAL;
        return -1;
    }

    (*src).fork(&mut *dst)
}
