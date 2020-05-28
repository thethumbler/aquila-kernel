use prelude::*;

use boot::*;
use sys::binfmt::elf::*;

use crate::arch::i386::cpu::init::virtual_address;
use crate::arch::i386::include::boot::multiboot::*;

#[inline]
unsafe fn get_multiboot_mmap_count(info: *const MultibootInfo) -> isize {
    let mut count = 0;
    let mut mmap_addr = (*info).mmap_addr as usize;
    let mmap_end = mmap_addr + (*info).mmap_length as usize;

    while mmap_addr < mmap_end {
        let mut mmap: *const MultibootMmapEntry = mmap_addr as *const MultibootMmapEntry;

        count += 1;
        mmap_addr += (*mmap).size as usize + core::mem::size_of::<u32>();
    }

    return count;
}

#[inline]
unsafe fn build_multiboot_mmap(info: *const MultibootInfo, boot_mmap: *mut BootMemoryMap) {
    let mut mmap_addr = (*info).mmap_addr as usize;
    let mmap_end  = mmap_addr + (*info).mmap_length as usize;
    let mut boot_mmap = boot_mmap;

    while mmap_addr < mmap_end {
        let mmap: *const MultibootMmapEntry = mmap_addr as *const MultibootMmapEntry;

        *boot_mmap = BootMemoryMap {
            map_type: BootMemoryMapType::from_u32((*mmap).map_type),
            start: (*mmap).addr as usize,
            end: ((*mmap).addr + (*mmap).len) as usize,
        };

        boot_mmap  = boot_mmap.offset(1);
        mmap_addr += (*mmap).size as usize + core::mem::size_of::<u32>();
    }
}

#[inline]
unsafe fn build_multiboot_modules(info: *const MultibootInfo, modules: *mut BootModule) {
    let mods_addr = (*info).mods_addr as usize;
    let mods = mods_addr as *const MultibootModList;

    for i in 0..(*info).mods_count {
        let module = &*mods.offset(i as isize);

        *modules.offset(i as isize) = BootModule {
            addr: virtual_address(module.mod_start as usize),
            size: (module.mod_end - module.mod_start) as usize,
            cmdline: virtual_address(module.cmdline as usize),
        };
    }
}

static mut boot_info: BootInfo = BootInfo::empty();
static mut boot_info_mmap: [BootMemoryMap; 32] = [BootMemoryMap::empty(); 32];
static mut boot_info_modules: [BootModule; 32] = [BootModule::empty(); 32];

pub unsafe fn process_multiboot_info(info: *const MultibootInfo) -> *const BootInfo {
    boot_info.cmdline = virtual_address((*info).cmdline as usize);
    boot_info.total_mem = ((*info).mem_lower + (*info).mem_upper) as usize;

    if (*info).flags & MULTIBOOT_INFO_ELF_SHDR != 0 {
        boot_info.shdr = virtual_address((*info).elf_sec.addr as usize);
        boot_info.shdr_num = (*info).elf_sec.num;

        let mut symtab: *mut Elf32SectionHeader = core::ptr::null_mut();
        let mut strtab: *mut Elf32SectionHeader = core::ptr::null_mut();

        for i in 0..boot_info.shdr_num {
            let shdr: *mut Elf32SectionHeader = boot_info.shdr.offset(i as isize) as *mut Elf32SectionHeader;

            if (*shdr).sh_type == SHT_SYMTAB {
                symtab = shdr;
                (*symtab).sh_addr = virtual_address((*symtab).sh_addr as usize) as *const u8 as usize as u32;
                boot_info.symtab = symtab;
                boot_info.symnum = (*shdr).sh_size / (core::mem::size_of::<Elf32Symbol>() as u32);
            }

            if (*shdr).sh_type == SHT_STRTAB && strtab.is_null() {
                strtab = shdr;
                (*strtab).sh_addr = virtual_address((*strtab).sh_addr as usize) as *const u8 as usize as u32;
                boot_info.strtab = strtab;
            }
        }
    }

    boot_info.mmap_count = get_multiboot_mmap_count(info);
    boot_info.mmap = boot_info_mmap.as_ptr();

    build_multiboot_mmap(info, boot_info_mmap.as_mut_ptr());

    /*
//#ifdef MULTIBOOT_GFX
    /* We report video memory as mmap region */
    boot_info.mmap[boot_info.mmap_count].type = MMAP_RESERVED;

    struct vbe_info_block  *vinfo = (struct vbe_info_block *)(uintptr_t)  info->vbe_control_info;
    struct mode_info_block *minfo = (struct mode_info_block *)(uintptr_t) info->vbe_mode_info;

    boot_info.mmap[boot_info.mmap_count].start = minfo->phys_base_ptr;
    boot_info.mmap[boot_info.mmap_count].end   = minfo->phys_base_ptr + minfo->y_resolution * minfo->lin_bytes_per_scanline;

    boot_info.mmap_count++;

    extern void earlycon_fb_register(uintptr_t, uint32_t, uint32_t, uint32_t, uint32_t);

    uintptr_t vaddr    = minfo->phys_base_ptr;
    uint32_t  scanline = minfo->lin_bytes_per_scanline;
    uint32_t  yres     = minfo->y_resolution;
    uint32_t  xres     = minfo->x_resolution;
    uint32_t  depth    = minfo->bits_per_pixel;

    //earlycon_fb_register(vaddr, scanline, yres, xres, depth);

    static struct fbdev_vesa data;
    data.vbe_info  = (struct vbe_info_block *) VMA(vinfo);
    data.mode_info = (struct mode_info_block *) VMA(minfo);

    /* And register fbdev of type `vesa' */
    //fbdev_register(FBDEV_TYPE_VESA, &data);
//#endif
    */

    boot_info.modules_count = (*info).mods_count as isize;
    boot_info.modules = boot_info_modules.as_ptr();

    build_multiboot_modules(info, boot_info_modules.as_mut_ptr());

    return &boot_info;
}
