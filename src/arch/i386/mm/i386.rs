use prelude::*;

use mm::*;
use crate::arch::i386::sys::execve::tlb_flush;
use crate::arch::i386::include::cpu::cpu::read_cr3;
use crate::arch::i386::cpu::init::{local_address, virtual_address};

use crate::{malloc_define, print, page_align};

malloc_define!(M_PMAP, "pmap\0", "physical memory map structure\0");

static mut cur_pmap: *mut PhysicalMap = core::ptr::null_mut();

#[repr(align(4096))]
struct PageTable([u32; 1024]);

static mut bootstrap_processor_table: *mut u32 = core::ptr::null_mut();
static mut last_page_table: PageTable = PageTable([0; 1024]);

#[repr(C)]
pub struct PhysicalMap {
    map: paddr_t,
    refcnt: size_t,
}

unsafe impl Sync for PhysicalMap {}

const MOUNT_ADDR: usize = 0xFFBFF000;

macro PAGE_DIR {
    ($i:expr) => {
        *(0xFFFFF000 as *mut u32).offset($i as isize)
    }
}

macro PAGE_TBL {
    ($i:expr, $j:expr) => {
        *((0xFFC00000 + 0x1000 * ($i)) as *mut u32).offset($j as isize)
    }
}


macro max {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    }
}

macro min {
    ($a:expr, $b:expr) => {
        if $a < $b { $a } else { $b }
    }
}

const PG_PRESENT: u32 = 1;
const PG_WRITE:   u32 = 2;
const PG_USER:    u32 = 4;

macro VTBL {
    ($n:expr) => {
        ($n >> 12) & 0x3ff
    }
}

macro VDIR {
    ($n:expr) => {
        ($n >> 22) & 0x3ff
    }
}

macro PHYSADDR {
    ($s:expr) => {
        $s & !0xfff
    }
}

unsafe fn copy_physical_to_virtual(virt_dest: usize, phys_src: paddr_t, n: size_t) -> usize {
    /* Copy up to page boundary */
    let offset = phys_src % PAGE_SIZE;
    let mut size = min!(n, PAGE_SIZE - offset);
    
    if size > 0 {
        let mut virt_dest = virt_dest;
        let mut phys_src = phys_src;
        let prev_mount = frame_mount(phys_src);
        let p = MOUNT_ADDR as *mut u8;
        memcpy(virt_dest as *mut u8, p.offset(offset as isize), size);
        
        phys_src  += size;
        virt_dest += size;

        /* copy complete pages */
        let n = n - size;
        size = n / PAGE_SIZE;
        while size != 0 {
            frame_mount(phys_src);
            memcpy(virt_dest as *mut u8, p, PAGE_SIZE);
            phys_src += PAGE_SIZE;
            virt_dest += PAGE_SIZE;
            size -= 1;
        }

        /* copy what is remainig */
        size = n % PAGE_SIZE;
        if size != 0 {
            frame_mount(phys_src);
            memcpy(virt_dest as *mut u8, p, size);
        }

        frame_mount(prev_mount);
    }

    return virt_dest;
}

unsafe fn copy_virtual_to_physical(phys_dest: paddr_t, virt_src: usize, n: size_t) -> usize {
    let mut size = n / PAGE_SIZE;
    let prev_mount = frame_mount(0);
    let mut phys_dest = phys_dest;
    let mut virt_src = virt_src;
    let ret = phys_dest;

    while size != 0 {
        frame_mount(phys_dest);
        let p = MOUNT_ADDR as *mut u8;
        memcpy(p, virt_src as *mut u8, PAGE_SIZE);
        phys_dest += PAGE_SIZE;
        virt_src  += PAGE_SIZE;
        size -= 1;
    }

    size = n % PAGE_SIZE;

    if size != 0 {
        frame_mount(phys_dest);
        let p = MOUNT_ADDR as *mut u8;
        memcpy(p, virt_src as *mut u8, size);
    }

    frame_mount(prev_mount);
    return ret;
}

unsafe fn tlb_invalidate_page(virt: usize) {
    asm!("invlpg (%eax)"::"{eax}"(virt));
}

unsafe fn frame_get() -> usize {
    let frame = buddy_alloc(BUDDY_ZONE_NORMAL, PAGE_SIZE);

    if frame == 0 {
        panic!("could not allocate frame");
    }

    let old = frame_mount(frame);
    let p = MOUNT_ADDR as *mut [u8; PAGE_SIZE];
    core::ptr::write(p, [0; PAGE_SIZE]);
    frame_mount(old);

    return frame;
}

unsafe fn frame_get_no_clr() -> usize {
    let frame = buddy_alloc(BUDDY_ZONE_NORMAL, PAGE_SIZE);

    if frame == 0 {
        panic!("could not allocate frame");
    }

    return frame;
}

unsafe fn frame_release(frame: usize) {
    buddy_free(BUDDY_ZONE_NORMAL, frame, PAGE_SIZE);
}

unsafe fn frame_mount(paddr: usize) -> usize {
    let mut prev = core::ptr::read_volatile((&last_page_table as *const _ as *mut u32).offset(1023)) as usize;
    prev &= !PAGE_MASK;

    if paddr == 0 {
        return prev;
    }

    if paddr & PAGE_MASK != 0 {
        panic!("mount must be on page (4K) boundary");
    }

    let page = paddr as u32 | PG_PRESENT | PG_WRITE;

    core::ptr::write_volatile((&last_page_table as *const _ as *mut u32).offset(1023), page);
    tlb_invalidate_page(MOUNT_ADDR);

    return prev;
}


/* ================== Table Helpers ================== */

unsafe fn table_alloc() -> paddr_t {
    let paddr = frame_get();

    let mut vm_page = mm_page(paddr);
    *vm_page = core::mem::zeroed();
    (*vm_page).paddr = paddr;

    return paddr;
}

unsafe fn table_dealloc(paddr: paddr_t) {
    frame_release(paddr);
}

unsafe fn table_map(paddr: paddr_t, pdidx: usize, flags: isize) -> isize {
    if pdidx > 1023 {
        return -EINVAL;
    }

    let table = paddr as u32 | (PG_PRESENT|PG_WRITE|PG_USER);
    PAGE_DIR!(pdidx) = table;

    tlb_flush();

    return 0;
}

unsafe fn table_unmap(pdidx: usize) {
    if pdidx > 1023 {
        return;
    }

    if PAGE_DIR!(pdidx) & PG_PRESENT != 0 {
        PAGE_DIR!(pdidx) &= !PG_PRESENT;
        table_dealloc(PAGE_DIR!(pdidx) as usize & !PAGE_MASK);
    }

    tlb_flush();
}

/* ================== Page Helpers ================== */
unsafe fn page_map(paddr: paddr_t, pdidx: usize, ptidx: usize, flags: u32) -> isize {
    /* Sanity checking */
    if pdidx > 1023 || ptidx > 1023 {
        return -EINVAL;
    }

    let mut page = paddr as u32 | PG_PRESENT;
    page |= if flags & (VM_KW | VM_UW) as u32 != 0 { PG_WRITE } else { 0 };
    page |= if flags & (VM_URWX) as u32 != 0 { PG_USER } else { 0 };

    /* check if table is present */
    if PAGE_DIR!(pdidx) & PG_PRESENT == 0 {
        let table = table_alloc();
        table_map(table, pdidx, flags as isize);
    }

    PAGE_TBL!(pdidx, ptidx) = page;

    /* Increment references to table */
    let table = PHYSADDR!(PAGE_DIR!(pdidx));
    mm_page_incref(table as usize);

    return 0;
}

unsafe fn page_protect(pdidx: usize, ptidx: usize, flags: u32) -> isize {
    /* sanity checking */
    if pdidx > 1023 || ptidx > 1023 {
        return -EINVAL;
    }

    /* check if table is present */
    if PAGE_DIR!(pdidx) & PG_PRESENT == 0 {
        return -EINVAL;
    }

    let mut page = PAGE_TBL!(pdidx, ptidx);

    if page & PG_PRESENT != 0 {
        page &= !(PG_WRITE|PG_USER);
        page |= if flags & (VM_KW | VM_UW) as u32 != 0 { PG_WRITE } else { 0 };
        page |= if flags & VM_URWX as u32 != 0 { PG_USER } else { 0 };

        PAGE_TBL!(pdidx, ptidx) = page;
    }

    return 0;
}

unsafe fn page_unmap(vaddr: usize) {
    if vaddr & PAGE_MASK != 0 {
        return;
    }

    let pdidx = VDIR!(vaddr);
    let ptidx = VTBL!(vaddr);

    if PAGE_DIR!(pdidx) & PG_PRESENT != 0 {
        if PAGE_TBL!(pdidx, ptidx) & PG_PRESENT != 0 {
            let old_page = page_align!(PAGE_TBL!(pdidx, ptidx));

            PAGE_TBL!(pdidx, ptidx) = 0;
            let table = PHYSADDR!(PAGE_DIR!(pdidx));

            mm_page_decref(table as usize);

            if mm_page_ref(table as usize) == 0 {
                table_unmap(pdidx);
            }

            tlb_invalidate_page(vaddr);
        }
    }
}

unsafe fn __page_get_mapping(vaddr: usize) -> u32 {
    if PAGE_DIR!(VDIR!(vaddr)) & PG_PRESENT != 0 {
        let page = PAGE_TBL!{ VDIR!(vaddr), VTBL!(vaddr) };

        if page & PG_PRESENT != 0 {
            return page;
        }
    }

    return 0;
}

pub unsafe fn pmap_switch(pmap: *mut PhysicalMap) -> *mut PhysicalMap {
    if pmap.is_null() {
        panic!("pmap?");
    }

    let ret = cur_pmap;

    if !cur_pmap.is_null() && (*cur_pmap).map == (*pmap).map {
        cur_pmap == pmap;
        return ret;
    }

    if !cur_pmap.is_null() {
        /* store current directory mapping in old_dir */
        copy_virtual_to_physical((*cur_pmap).map, bootstrap_processor_table as usize, 768 * 4);
    }

    copy_physical_to_virtual(bootstrap_processor_table as usize, (*pmap).map, 768 * 4);
    cur_pmap = pmap;
    tlb_flush();

    return ret;
}

static mut k_pmap: PhysicalMap = PhysicalMap { map: 0, refcnt: 0 };

unsafe fn setup_i386_paging() {
    print!("x86: setting up 32-bit paging\n");
    let __cur_pd = read_cr3() & !PAGE_MASK;

    bootstrap_processor_table = virtual_address(__cur_pd);
    *bootstrap_processor_table.offset(1023) = local_address(bootstrap_processor_table as usize) as *const u8 as usize as u32 | PG_WRITE | PG_PRESENT;
    *bootstrap_processor_table.offset(1022) = local_address(&last_page_table as *const _ as usize) as *const u8 as usize as u32 | PG_PRESENT | PG_WRITE;

    /* unmap lower half */
    let mut i = 0;
    while *bootstrap_processor_table.offset(i) != 0 {
        *bootstrap_processor_table.offset(i) = 0;
        i += 1;
    }

    tlb_flush();

    k_pmap.map = __cur_pd;
    kvm_space.pmap = &mut k_pmap;
}

/*
 *  Archeticture Interface
 */

pub unsafe fn arch_page_unmap(vaddr: usize) -> isize {
    if vaddr & PAGE_MASK != 0 {
        return -EINVAL;
    }

    page_unmap(vaddr);
    return 0;
}

pub unsafe fn arch_mm_page_fault(vaddr: usize, err: usize) {
    let mut flags = 0;

    flags |= if err & 0x01 != 0 { PF_PRESENT } else { 0 };
    flags |= if err & 0x02 != 0 { PF_WRITE   } else { PF_READ };
    flags |= if err & 0x04 != 0 { PF_USER    } else { 0 };
    flags |= if err & 0x10 != 0 { PF_EXEC    } else { 0 };

    mm_page_fault(vaddr, flags as isize);

    return;
}

unsafe fn pmap_alloc() -> *mut PhysicalMap {
    return kmalloc(core::mem::size_of::<PhysicalMap>(), &M_PMAP, M_ZERO) as *mut PhysicalMap;
}

unsafe fn pmap_release(pmap: *mut PhysicalMap) {
    if pmap == cur_pmap {
        pmap_switch(&mut k_pmap);
    }

    if (*pmap).map != 0 {
        frame_release((*pmap).map);
    }

    kfree(pmap as *mut u8);
}

pub unsafe fn pmap_init() {
    setup_i386_paging();
    cur_pmap = &mut k_pmap;
}

pub unsafe fn pmap_create() -> *mut PhysicalMap {
    let pmap = pmap_alloc();

    if pmap.is_null() {
        return core::ptr::null_mut();
    }

    (*pmap).map = frame_get();
    (*pmap).refcnt = 1;

    return pmap;
}

pub unsafe fn pmap_incref(pmap: *mut PhysicalMap) {
    /* XXX Handle overflow */
    (*pmap).refcnt += 1;
}

pub unsafe fn pmap_decref(pmap: *mut PhysicalMap) {
    (*pmap).refcnt -= 1;

    if (*pmap).refcnt == 0 {
        pmap_release(pmap);
    }
}

pub unsafe fn pmap_add(pmap: *mut PhysicalMap, va: usize, pa: usize, flags: u32) -> isize {
    if va & PAGE_MASK != 0 || pa & PAGE_MASK != 0 {
        return -EINVAL;
    }

    let old_map = pmap_switch(pmap);

    let pdidx = VDIR!(va);
    let ptidx = VTBL!(va);

    page_map(pa, pdidx, ptidx, flags);
    tlb_invalidate_page(va);

    pmap_switch(old_map);

    return 0;
}

pub unsafe fn pmap_remove(pmap: *mut PhysicalMap, mut sva: usize, eva: usize) {
    if sva & PAGE_MASK !=  0 || eva & PAGE_MASK != 0 {
        return;
    }

    let old_map = pmap_switch(pmap);

    while sva < eva {
        page_unmap(sva);
        sva += PAGE_SIZE;
    }

    pmap_switch(old_map);
}

unsafe fn table_remove_all(table: paddr_t) {
    let mnt = frame_mount(table);
    let pages = MOUNT_ADDR as *mut u32;

    for i in 0..1024 {
        if *pages.offset(i) & PG_PRESENT != 0 {
            let page = PHYSADDR!(*pages.offset(i));
            *pages.offset(i) = 0;

            mm_page_decref(page as usize);
            mm_page_decref(table);
        }
    }

    frame_mount(mnt);
}

pub unsafe fn pmap_remove_all(pmap: *mut PhysicalMap) {
    let old_map = pmap_switch(&mut k_pmap);
    let base = (*pmap).map;

    let old_mount = frame_mount(base);
    let tbl = MOUNT_ADDR as *mut u32;

    for i in 0..768 {
        if *tbl.offset(i) & PG_PRESENT != 0 {
            let table = PHYSADDR!(*tbl.offset(i));
            table_remove_all(table as paddr_t);
            *tbl.offset(i) = 0;
            table_dealloc(table as paddr_t);
        }
    }

    tlb_flush();
    frame_mount(old_mount);
    pmap_switch(old_map);

    return;
}

pub unsafe fn pmap_protect(pmap: *mut PhysicalMap, mut sva: vaddr_t, eva: vaddr_t, prot: u32) {
    if sva & PAGE_MASK != 0 {
        return;
    }

    let old_map = pmap_switch(pmap);

    while sva < eva {
        let pdidx = VDIR!(sva);
        let ptidx = VTBL!(sva);

        page_protect(pdidx, ptidx, prot);
        tlb_invalidate_page(sva);

        sva += PAGE_SIZE;
    }

    pmap_switch(old_map);
}

pub unsafe fn pmap_copy(dst_map: *mut PhysicalMap, src_map: *mut PhysicalMap, dst_addr: vaddr_t, len: size_t, src_addr: vaddr_t) {
    return;
}

pub unsafe fn pmap_update(pmap: *mut PhysicalMap) {
    return;
}

struct CopyBuffer([u8; PAGE_SIZE]);
static mut __copy_buf: CopyBuffer = CopyBuffer([0; PAGE_SIZE]);

pub unsafe fn pmap_page_copy(src: paddr_t, dst: paddr_t) {
    copy_physical_to_virtual(&__copy_buf as *const _ as usize, src, PAGE_SIZE);
    copy_virtual_to_physical(dst, &__copy_buf as *const _ as usize, PAGE_SIZE);
    return;
}

pub unsafe fn pmap_page_protect(pg: *mut VmPage, flags: u32) {
    return;
}

pub unsafe fn pmap_clear_modify(pg: *mut VmPage) ->  isize {
    return -1;
}

pub unsafe fn pmap_clear_reference(pg: *mut VmPage) -> isize {
    return -1;
}

pub unsafe fn pmap_is_modified(pg: *mut VmPage) -> isize {
    return -1;
}

pub unsafe fn pmap_is_referenced(pg: *mut VmPage) -> isize {
    return -1;
}

pub unsafe fn arch_page_get_mapping(pmap: *mut PhysicalMap, vaddr: vaddr_t) -> paddr_t {
    let old_map = pmap_switch(pmap);

    let page = __page_get_mapping(vaddr);

    if page != 0 {
        pmap_switch(old_map);
        return PHYSADDR!(page as usize);
    }

    pmap_switch(old_map);

    return 0;
}

pub unsafe fn pmap_page_read(paddr: paddr_t, off: usize, size: usize, buf: *mut u8) -> isize {
    let sz = max!(PAGE_SIZE, size - off);
    let old = frame_mount(paddr);
    let page = MOUNT_ADDR as *const u8;
    memcpy(buf, page.offset(off as isize), sz);
    frame_mount(old);
    return 0;
}

pub unsafe fn pmap_page_write(paddr: paddr_t, off: usize, size: usize, buf: *const u8) -> isize {
    let sz = max!(PAGE_SIZE, size - off);
    let old = frame_mount(paddr);
    let page = MOUNT_ADDR as *mut u8;
    memcpy(page.offset(off as isize), buf, sz);
    frame_mount(old);
    return 0;
}
