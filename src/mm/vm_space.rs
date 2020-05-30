use prelude::*;

use mm::*;
use arch::mm::i386::*;

#[derive(Debug)]
pub struct VmSpace {
    /** physical memory mapper (arch-specific) */
    pub pmap: *mut PhysicalMap,

    /** virtual memory regions inside the vm space */
    pub vm_entries: Queue<*mut VmEntry>,
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

    pub fn destroy(&mut self) {
        unsafe {
            let vm_entries = &mut self.vm_entries;
            let mut vm_entry = vm_entries.dequeue();

            while !vm_entry.is_none() {
                (*vm_entry.unwrap()).destroy();
                Box::from_raw(vm_entry.unwrap());
                vm_entry = (*vm_entries).dequeue();
            }

            pmap_remove_all(self.pmap);
        }
    }

    pub fn fork(&mut self, dst: &mut VmSpace) -> isize {
        /* copy vm entries */
        let src_vm_entries = &mut self.vm_entries;

        for qnode in src_vm_entries.iter() {
            unsafe {
                let s_entry = &*(qnode.value as *mut VmEntry);
                let mut d_entry = Box::leak(VmEntry::alloc());

                *d_entry = *s_entry;
                (*d_entry).qnode = (*dst).vm_entries.enqueue(&mut *d_entry);

                if !s_entry.vm_anon.is_null() {
                    let vm_anon = &mut *s_entry.vm_anon;

                    vm_anon.flags |= VM_COPY;
                    vm_anon.incref();
                }

                if !s_entry.vm_object.is_null() {
                    let vm_object = &mut *s_entry.vm_object;
                    vm_object.refcnt += 1;
                }

                if (s_entry.flags & (VM_UW|VM_KW)) != 0 && (s_entry.flags & VM_SHARED) == 0 {
                    /* remove write permission from all pages */
                    let sva = s_entry.base;
                    let eva = sva + s_entry.size;
                    let flags = s_entry.flags & !(VM_UW|VM_KW);

                    pmap_protect(self.pmap, sva, eva, flags as u32);
                }
            }
        }

        return 0;
    }
}
