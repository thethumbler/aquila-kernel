use prelude::*;

use arch::i386::mm::i386::*;
use arch::i386::mm::mm::arch_mm_setup;
use boot::*;
use mm::*;
use sys::sched::*;

/* FIXME use boot time allocation scheme */
pub static mut PAGES: [VmPage; 768*1024] = [VmPage {
    paddr: 0,
    vm_object: core::ptr::null_mut(),
    off: 0,
    refcnt: 0,
}; 768*1024];

macro_rules! page {
    ($addr:expr) => {
        (PAGES[(($addr)/PAGE_SIZE) as usize])
    }
}

#[inline(always)]
pub unsafe fn mm_page_incref(paddr: paddr_t) -> () {
    page!(paddr).refcnt += 1;
}

#[inline(always)]
pub unsafe fn mm_page_decref(paddr: paddr_t) -> () {
    page!(paddr).refcnt -= 1;
}

#[inline(always)]
pub unsafe fn mm_page_ref(paddr: paddr_t) -> usize {
    return page!(paddr).refcnt;
}

#[inline(always)]
pub unsafe fn mm_page(paddr: paddr_t) -> *mut VmPage {
    return &mut page!(paddr);
}

#[inline(always)]
pub unsafe fn mm_page_alloc() -> *mut VmPage {
    /* Get new frame */
    let paddr = buddy_alloc(BUDDY_ZONE_NORMAL, PAGE_SIZE as usize);
    let vm_page = &mut page!(paddr);

    //core::ptr::write_bytes(vm_page, 0, core::mem::size_of::<VmPage>());
    *vm_page = VmPage { 
        paddr: 0,
        vm_object: core::ptr::null_mut(),
        off: 0,
        refcnt: 0,
    };

    (*vm_page).paddr = paddr;

    return vm_page;
}

pub unsafe fn mm_page_dealloc(paddr: paddr_t) -> () {
    /* TODO: Check out of bounds */
    buddy_free(BUDDY_ZONE_NORMAL, paddr, PAGE_SIZE as usize);

    /* Release frame if it is no longer referenced */
    //if (mm_page_ref(paddr) == 0)
    //    buddy_free(BUDDY_ZONE_NORMAL, paddr, PAGE_SIZE);
}

pub unsafe fn mm_page_map(pmap: *mut PhysicalMap, vaddr: usize, paddr: paddr_t, flags: isize) -> isize {
    /* TODO: Check out of bounds */

    /* Increment references count to physical page */
    //mm_page_incref(paddr);

    return pmap_add(pmap, vaddr, paddr, flags as u32);
}

pub unsafe fn mm_page_unmap(pmap: *mut PhysicalMap, vaddr: usize) -> isize {
    //printk("mm_page_unmap(pmap=%p, vaddr=%p)\n", pmap, vaddr);

    /* TODO: Check out of bounds */

    /* Check if page is mapped */
    let paddr = arch_page_get_mapping(pmap, vaddr);

    if paddr != 0 {
        /* Decrement references count to physical page */
        //mm_page_decref(paddr);

        /* Call arch specific page unmapper */
        pmap_remove(pmap, vaddr, vaddr + PAGE_SIZE);

        /* Release page -- checks ref count */
        //mm_page_dealloc(paddr);

        return 0;
    }

    //return -EINVAL;
    return -1;
}

pub unsafe fn mm_map(pmap: *mut PhysicalMap, paddr: paddr_t, vaddr: usize, size: usize, flags: isize) -> isize {
    //printk(b"mm_map(pmap=%p, paddr=%p, vaddr=%p, size=%d, flags=%x)\n\0".as_ptr(), pmap, paddr, vaddr, size, flags);

    /* TODO: Check out of bounds */

    let alloc = paddr == 0;

    let endaddr   = page_round!(vaddr + size);
    let mut vaddr = page_align!(vaddr);
    let mut paddr = page_align!(paddr);

    let mut nr = (endaddr - vaddr) / PAGE_SIZE;

    while nr > 0 {
        let phys = arch_page_get_mapping(pmap, vaddr);

        if phys == 0 {
            if alloc {
                paddr = (*mm_page_alloc()).paddr;
                //printk("paddr = %p\n", paddr);
            }

            mm_page_map(pmap, vaddr, paddr, flags);
        }

        vaddr += PAGE_SIZE;
        paddr += PAGE_SIZE;

        nr -= 1;
    }

    return 0;
}

pub unsafe fn mm_unmap(pmap: *mut PhysicalMap, vaddr: usize, size: usize) -> () {
    //printk("mm_unmap(pmap=%p, vaddr=%p, size=%ld)\n", pmap, vaddr, size);
    /* TODO: Check out of bounds */

    if size < PAGE_SIZE as usize {
        return;
    }

    let mut sva = page_round!(vaddr);
    let eva = page_align!(vaddr + size);

    //pmap_remove(pmap, sva, eva);

    let mut nr = (eva - sva)/PAGE_SIZE;

    while nr > 0 {
        mm_page_unmap(pmap, sva);
        sva += PAGE_SIZE;
        nr -= 1;
    }
}

pub unsafe fn mm_unmap_full(pmap: *mut PhysicalMap, vaddr: usize, size: usize) -> () {
    //printk("mm_unmap_full(pmap=%p, vaddr=%p, size=%ld)\n", pmap, vaddr, size);

    /* TODO: Check out of bounds */

    let mut start = page_align!(vaddr);
    let end = page_round!(vaddr + size);

    let mut nr = (end - start)/PAGE_SIZE;

    while nr > 0 {
        mm_page_unmap(pmap, start);
        start += PAGE_SIZE;
        nr -= 1;
    }
}

extern "C" {
    static _VMA: u8;
}

macro_rules! physical_address {
    ($obj:expr) => {
        (($obj) as usize - (&_VMA) as *const _ as usize)
    }
}

pub unsafe fn mm_setup(boot: *mut BootInfo) {
    print!("kernel: total memory: {} KiB, {} MiB\n", (*boot).total_mem, (*boot).total_mem / 1024);

    buddy_setup((*boot).total_mem * 1024);

    /* Setup memory regions */
    for i in 0..(*boot).mmap_count {
        if (*(*boot).mmap.offset(i)).map_type == BootMemoryMapType::MMAP_RESERVED {
            let mmap = (*boot).mmap.offset(i);
            let size = (*mmap).end - (*mmap).start;
            buddy_set_unusable((*mmap).start, size);
        }
    }

    /* Mark modules space as unusable */
    for i in 0..(*boot).modules_count {
        let module = (*boot).modules.offset(i);
        buddy_set_unusable(physical_address!((*module).addr), (*module).size);
    }

    arch_mm_setup();
}

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_MASK: usize = 4096 - 1;

pub macro page_align {
    ($ptr:expr) => {
        (($ptr as usize) & !PAGE_MASK)
    }
}

pub macro page_round {
    ($ptr:expr) => {
        ((($ptr as usize) + PAGE_MASK) & !PAGE_MASK)
    }
}

pub const PF_PRESENT: usize = 0x001;
pub const PF_READ:    usize = 0x002;
pub const PF_WRITE:   usize = 0x004;
pub const PF_EXEC:    usize = 0x008;
pub const PF_USER:    usize = 0x010;
