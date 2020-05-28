use prelude::*;
use mm::*;

pub const BUDDY_MAX_ORDER: usize = 10;
pub const BUDDY_MIN_BS:    usize = 4096;
pub const BUDDY_MAX_BS:    usize = BUDDY_MIN_BS << BUDDY_MAX_ORDER;

pub const BUDDY_ZONE_NR:     usize = 2;
pub const BUDDY_ZONE_DMA:    usize = 0;
pub const BUDDY_ZONE_NORMAL: usize = 1;

use crate::{buddy_idx, print};

#[no_mangle]
static mut k_total_mem: usize = 0;

#[no_mangle]
static mut k_used_mem: usize = 0;

static mut kstart: usize = 0;
static mut kend: usize = 0;

const ALLOC_AREA_SIZE: usize = 1024 * 1024;

#[no_mangle]
pub static mut alloc_area: [u8; ALLOC_AREA_SIZE] = [0; ALLOC_AREA_SIZE]; /* 1 MiB heap area */

#[no_mangle]
pub static mut alloc_mark: *mut u8 = core::ptr::null_mut(); //&mut alloc_area as *mut u8;

pub unsafe fn alloc(size: usize, align: usize) -> *mut u8 {
    //char *ret = (char *)((uintptr_t)(alloc_mark + align - 1) & (~(align - 1)));

    let ret = ((alloc_mark.offset((align - 1) as isize) as usize) & (!(align - 1))) as *const u8 as *mut u8;

    alloc_mark = ret.offset(size as isize);
    //memset(ret, 0, size);

    core::ptr::write_bytes(ret, 0, size);
    
    return ret;
}

#[no_mangle]
pub static mut buddies: [[Buddy; BUDDY_MAX_ORDER+1]; BUDDY_ZONE_NR] = [
    [
        Buddy {
            first_free_idx: 0,
            usable: 0,
            bitmap: BitMap {
                map: core::ptr::null_mut(),
                max_idx: 0,
            }
        }; BUDDY_MAX_ORDER+1
    ]; BUDDY_ZONE_NR
];

#[no_mangle]
pub static buddy_zone_offset: [usize; BUDDY_ZONE_NR+1] = [
    //[BUDDY_ZONE_DMA]     = 0,           /* 0 - 16 MiB */
    //[BUDDY_ZONE_NORMAL]  = 0x1000000,   /* 16 MiB -  */
    //[BUDDY_ZONE_NR]      = (uintptr_t) -1,
    0,
    0x1000000,
    !0
];

pub unsafe fn buddy_recursive_alloc(zone: usize, order: usize) -> usize {
    if order > BUDDY_MAX_ORDER {
        return (-1isize) as usize;
    }

    /* Check if there is a free bit in current order */
    if buddies[zone][order].usable != 0 {
        /* Search for a free bit in current order */
        for i in buddies[zone][order].first_free_idx..buddies[zone][order].bitmap.max_idx + 1 {

            /* If bit at i is not checked */
            if bitmap_check(&mut buddies[zone][order].bitmap, i) == 0 {

                /* Mark the bit as used */
                bitmap_set(&mut buddies[zone][order].bitmap, i);
                buddies[zone][order].usable -= 1;

                /* Shift first_free_idx to search after child_idx */
                buddies[zone][order].first_free_idx = i + 1;

                return i;
            }                
        }

        return (-1isize) as usize;
    } else {
        /* Search for a buddy in higher order to split */
        let idx = buddy_recursive_alloc(zone, order + 1);

        /* Could not find a free budy */
        if idx == (-1isize) as usize {
            return (-1isize) as usize;
        }

        /* Select the left child of the selected bit */
        let child_idx = idx << 1;

        /* Mark the selected bit as used */
        bitmap_set(&mut buddies[zone][order].bitmap, child_idx);

        /* Mark it's buddy as free */
        bitmap_clear(&mut buddies[zone][order].bitmap, buddy_idx!(child_idx));
        buddies[zone][order].usable += 1;

        /* Shift first_free_idx to search after child_idx */
        buddies[zone][order].first_free_idx = child_idx + 1;

        return child_idx;
    }
}

pub unsafe fn buddy_recursive_free(zone: usize, order: usize, idx: usize) {
    if order > BUDDY_MAX_ORDER {
        return;
    }

    if idx > buddies[zone][order].bitmap.max_idx {
        return;
    }

    /* Can't free an already free bit */
    if bitmap_check(&mut buddies[zone][order].bitmap, idx) == 0 {
        return;
    }

    /* Check if buddy bit is free, then combine */
    if order < BUDDY_MAX_ORDER && bitmap_check(&mut buddies[zone][order].bitmap, buddy_idx!(idx)) == 0 {
        bitmap_set(&mut buddies[zone][order].bitmap, buddy_idx!(idx));
        buddies[zone][order].usable -= 1;

        buddy_recursive_free(zone, order + 1, idx >> 1);
    } else {
        bitmap_clear(&mut buddies[zone][order].bitmap, idx);
        buddies[zone][order].usable += 1;

        /* Update first_free_idx */
        if buddies[zone][order].first_free_idx > idx {
            buddies[zone][order].first_free_idx = idx;
        }
    }
}

/* allocate new buddy
 * @param zone zone index
 * @param _sz chunk size
 */
pub unsafe fn buddy_alloc(zone: usize, _sz: usize) -> paddr_t {
    if _sz > BUDDY_MAX_BS {
        panic!("Cannot allocate buddy");
    }

    let mut sz = BUDDY_MIN_BS;

    /* FIXME */
    let mut order = 0;

    while order <= BUDDY_MAX_ORDER {
        if sz >= _sz {
            break;
        }

        sz <<= 1;
        order += 1;
    }

    k_used_mem += sz;

    let idx = buddy_recursive_alloc(zone, order);

    if idx != (-1isize) as usize {
        return buddy_zone_offset[zone] + (idx * (BUDDY_MIN_BS << order));
    } else {
        panic!("Cannot find free buddy");
        //return (uintptr_t) NULL;
    }
}

pub unsafe fn buddy_free(zone: usize, addr: paddr_t, size: usize) {
    let mut addr = addr as usize;

    if addr >= kstart && addr < kend {
        panic!("trying to free from kernel code");
    }

    let mut sz = BUDDY_MIN_BS;

    /* FIXME */
    let mut order = 0;
    while order < BUDDY_MAX_ORDER {
        if sz >= size {
            break;
        }

        sz <<= 1;
        order += 1;
    }

    k_used_mem -= sz;

    addr -= buddy_zone_offset[zone];
    let idx = ((addr as usize) / (BUDDY_MIN_BS << order));  // & (sz - 1);
    buddy_recursive_free(zone, order, idx);
}

macro_rules! max {
    ($a:expr, $b:expr) => {
        if ($a) > ($b) { $a } else { $b }
    }
}

macro_rules! min {
    ($a:expr, $b:expr) => {
        if ($a) < ($b) { $a } else { $b }
    }
}

#[no_mangle]
pub unsafe fn buddy_set_unusable(addr: usize, size: usize) {
    let addr = addr as usize;

    print!("buddy: set unusable: {:p}-{:p}\n", addr as *const u8, (addr+size-1) as *const u8);

    for zone in 0..BUDDY_ZONE_NR {

        let zone_start = buddy_zone_offset[zone];
        let zone_end   = buddy_zone_offset[zone+1];

        if addr + size < zone_start || addr >= zone_end {
            continue;
        }

        let zone_addr = max!(addr, zone_start) - zone_start;
        let zone_size = min!(addr + size, zone_end) - zone_start - zone_addr;

        let start_idx = zone_addr / BUDDY_MAX_BS;
        let mut end_idx   = (zone_addr + zone_size + BUDDY_MAX_BS - 1) / BUDDY_MAX_BS;

        if end_idx > buddies[zone][BUDDY_MAX_ORDER].bitmap.max_idx {
            end_idx = buddies[zone][BUDDY_MAX_ORDER].bitmap.max_idx;
        }

        bitmap_set_range(&mut buddies[zone][BUDDY_MAX_ORDER].bitmap, start_idx, end_idx);

        let ffidx = buddies[zone][BUDDY_MAX_ORDER].first_free_idx;

        if ffidx >= start_idx && ffidx <= end_idx {
            buddies[zone][BUDDY_MAX_ORDER].first_free_idx = end_idx + 1;
        }

        buddies[zone][BUDDY_MAX_ORDER].usable -= (end_idx - start_idx + 1);

        k_used_mem += size;
    }
}

pub unsafe fn buddy_setup(total_mem: usize) -> isize {
    print!("buddy: Setting up buddy allocator (total memory {:#x})\n", total_mem);

    alloc_mark = &mut alloc_area as *const _ as *mut u8;

    k_total_mem = total_mem;
    k_used_mem  = 0;

    for zone in 0..BUDDY_ZONE_NR {
        let mut bits_cnt = 0;

        if zone < BUDDY_ZONE_NR - 1 {
            bits_cnt = (buddy_zone_offset[zone+1] - buddy_zone_offset[zone]) / BUDDY_MAX_BS;
        } else {
            /* Last zone */
            bits_cnt = (total_mem - buddy_zone_offset[zone]) / BUDDY_MAX_BS;
        }

        for i in (0..BUDDY_MAX_ORDER+1).rev() {
            let bmsize = bitmap_size(bits_cnt);

            buddies[zone][i].bitmap.map = alloc(bmsize, 4) as *mut u32;
            buddies[zone][i].bitmap.max_idx = bits_cnt - 1;

            bits_cnt <<= 1;
        }

        /* Set the heighst order as free and the rest as unusable */
        bitmap_clear_range(&mut buddies[zone][BUDDY_MAX_ORDER].bitmap, 0, buddies[zone][BUDDY_MAX_ORDER].bitmap.max_idx);
        buddies[zone][BUDDY_MAX_ORDER].first_free_idx = 0;
        buddies[zone][BUDDY_MAX_ORDER].usable = buddies[zone][BUDDY_MAX_ORDER].bitmap.max_idx + 1;

        for i in 0..BUDDY_MAX_ORDER {
            bitmap_set_range(&mut buddies[zone][i].bitmap, 0, buddies[zone][i].bitmap.max_idx);
            buddies[zone][i].first_free_idx = (-1isize) as usize;
            buddies[zone][i].usable = 0;
        }
    }

    /* FIXME */
    extern "C" {
        static kernel_start: u8;
        static kernel_end: u8;
    }

    kstart = &kernel_start as *const _ as usize;
    kend   = &kernel_end as *const _ as usize;

    buddy_set_unusable(kstart, kend - kstart);

    //extern struct boot *__kboot;

    //if (__kboot->symtab) {
    //    struct elf32_shdr *shdr = __kboot->symtab;
    //    buddy_set_unusable(LMA((uintptr_t) shdr->sh_addr), shdr->sh_size);
    //}

    //if (__kboot->strtab) {
    //    struct elf32_shdr *shdr = __kboot->strtab;
    //    buddy_set_unusable(LMA((uintptr_t) shdr->sh_addr), shdr->sh_size);
    //}

    return 0;
}
