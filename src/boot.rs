use prelude::*;

use crate::sys::binfmt::elf::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BootModule {
    pub addr: *const u8,
    pub size: usize,
    pub cmdline: *const u8,
}

impl BootModule {
    pub const fn empty() -> Self {
        BootModule {
            addr: core::ptr::null(),
            size: 0,
            cmdline: core::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(PartialEq, Copy, Clone)]
pub enum BootMemoryMapType {
    MMAP_INVALID  = 0,
    MMAP_USABLE   = 1,
    MMAP_RESERVED = 2
}

impl BootMemoryMapType {
    pub fn from_u32(i: u32) -> Self {
        match i {
            1 => BootMemoryMapType::MMAP_USABLE,
            2 => BootMemoryMapType::MMAP_RESERVED,
            _ => BootMemoryMapType::MMAP_INVALID,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BootMemoryMap {
    pub map_type: BootMemoryMapType,
    pub start: usize,
    pub end: usize,
}

impl BootMemoryMap {
    pub const fn empty() -> Self {
        BootMemoryMap {
            map_type: BootMemoryMapType::MMAP_USABLE,
            start: 0,
            end: 0,
        }
    }
}

#[repr(C)]
pub struct BootInfo {
    pub cmdline: *const u8,
    pub total_mem: usize,

    pub modules_count: isize,
    pub mmap_count: isize,
    pub modules: *const BootModule,
    pub mmap: *const BootMemoryMap,

    pub shdr: *const Elf32SectionHeader,
    pub shdr_num: u32,

    pub symtab: *const Elf32SectionHeader,
    pub symnum: u32,

    pub strtab: *const Elf32SectionHeader,
}

impl BootInfo {
    pub const fn empty() -> Self {
        BootInfo {
            cmdline: core::ptr::null(),
            total_mem: 0,

            modules_count: 0,
            mmap_count: 0,
            modules: core::ptr::null(),
            mmap: core::ptr::null(),

            shdr: core::ptr::null(),
            shdr_num: 0,

            symtab: core::ptr::null(),
            symnum: 0,

            strtab: core::ptr::null(),
        }
    }
}
