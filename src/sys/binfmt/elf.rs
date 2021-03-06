use prelude::*;
use fs::{self, *};
use sys::process::*;
use mm::*;


use crate::{page_align, page_round};

type Elf32Sword = i32;
type Elf32Word  = u32;
type Elf32Addr  = u32; 
type Elf32Off   = u32; 
type Elf32Half  = u16; 

pub const ET_NONE     : usize = 0x0000;
pub const ET_REL      : usize = 0x0001;
pub const ET_EXEC     : usize = 0x0002;
pub const ET_DYN      : usize = 0x0003;
pub const ET_CORE     : usize = 0x0004;
pub const ET_LOPROC   : usize = 0xff00;
pub const ET_HIPROC   : usize = 0xffff;

pub const EM_NONE     : usize = 0x0000;
pub const EM_M32      : usize = 0x0001;
pub const EM_SPARC    : usize = 0x0002;
pub const EM_386      : usize = 0x0003;
pub const EM_68K      : usize = 0x0004;
pub const EM_88K      : usize = 0x0005;
pub const EM_860      : usize = 0x0007;
pub const EM_MIPS     : usize = 0x0008;

pub const EV_NONE     : usize = 0x0000;
pub const EV_CURRENT  : usize = 0x0001;

pub const EI_MAG0     : usize = 0;
pub const EI_MAG1     : usize = 1;
pub const EI_MAG2     : usize = 2;
pub const EI_MAG3     : usize = 3;
pub const EI_CLASS    : usize = 4;
pub const EI_DATA     : usize = 5;
pub const EI_VERSION  : usize = 6;
pub const EI_PAD      : usize = 7;
pub const EI_NIDENT   : usize = 16;

pub const ELFMAG0     : u8 = 0x7f;
pub const ELFMAG1     : u8 = b'E';
pub const ELFMAG2     : u8 = b'L';
pub const ELFMAG3     : u8 = b'F';

pub const ELFCLASSNONE    : u8 = 0;
pub const ELFCLASS32      : u8 = 1;
pub const ELFCLASS64      : u8 = 2;

pub const SHN_UNDEF       : usize = 0x0000;
pub const SHN_LORESERVE   : usize = 0xff00;
pub const SHN_LOPROC      : usize = 0xff00;
pub const SHN_HIPROC      : usize = 0xff1f;
pub const SHN_ABS         : usize = 0xfff1;
pub const SHN_COMMON      : usize = 0xfff2;
pub const SHN_HIRESERVE   : usize = 0xffff;

pub const SHT_NULL        : Elf32Word = 0;
pub const SHT_PROGBITS    : Elf32Word = 1;
pub const SHT_SYMTAB      : Elf32Word = 2;
pub const SHT_STRTAB      : Elf32Word = 3;
pub const SHT_RELA        : Elf32Word = 4;
pub const SHT_HASH        : Elf32Word = 5;
pub const SHT_DYNAMIC     : Elf32Word = 6;
pub const SHT_NOTE        : Elf32Word = 7;
pub const SHT_NOBITS      : Elf32Word = 8;
pub const SHT_REL         : Elf32Word = 9;
pub const SHT_SHLIB       : Elf32Word = 10;
pub const SHT_DYNSYM      : Elf32Word = 11;
pub const SHT_LOPROC      : Elf32Word = 0x70000000;
pub const SHT_HIPROC      : Elf32Word = 0x7fffffff;
pub const SHT_LOUSER      : Elf32Word = 0x80000000;
pub const SHT_HIUSER      : Elf32Word = 0xffffffff;

pub const DT_NULL     : usize = 0;
pub const DT_NEEDED   : usize = 1;
pub const DT_PLTRELSZ : usize = 2;
pub const DT_PLTGOT   : usize = 3;
pub const DT_HASH     : usize = 4;
pub const DT_STRTAB   : usize = 5;
pub const DT_SYMTAB   : usize = 6;
pub const DT_RELA     : usize = 7;
pub const DT_RELASZ   : usize = 8;
pub const DT_RELAENT  : usize = 9;
pub const DT_STRSZ    : usize = 10;
pub const DT_SYMENT   : usize = 11;
pub const DT_INIT     : usize = 12;
pub const DT_FINI     : usize = 13;
pub const DT_SONAME   : usize = 14;
pub const DT_RPATH    : usize = 15;
pub const DT_SYMBOLIC : usize = 16;
pub const DT_REL      : usize = 17;
pub const DT_RELSZ    : usize = 18;
pub const DT_RELENT   : usize = 19;
pub const DT_PLTREL   : usize = 20;
pub const DT_DEBUG    : usize = 21;
pub const DT_TEXTREL  : usize = 22;
pub const DT_JMPREL   : usize = 23;
pub const DT_LOPROC   : usize = 0x70000000;
pub const DT_HIPROC   : usize = 0x7fffffff;

pub const PT_NULL     : usize = 0;
pub const PT_LOAD     : usize = 1;
pub const PT_DYNAMIC  : usize = 2;
pub const PT_INTERP   : usize = 3;
pub const PT_NOTE     : usize = 4;
pub const PT_SHLIB    : usize = 5;
pub const PT_PHDR     : usize = 6;
pub const PT_LOPROC   : usize = 0x70000000;
pub const PT_HIPROC   : usize = 0x7fffffff;

pub const PF_X        : usize = 0x1;
pub const PF_W        : usize = 0x2;
pub const PF_R        : usize = 0x4;
pub const PF_MASKPROC : usize = 0xf0000000;

pub fn elf32_st_bind(st_info: u8) -> u8 {
    st_info >> 4
}

pub fn elf32_st_type(st_info: u8) -> u8 {
    st_info & 0xf
}

pub fn elf32_st_info(bind: u8, r#type: u8) -> u8 {
    (bind << 4) + (r#type & 0xf)
}

pub const STB_LOCAL   : usize = 0;
pub const STB_GLOBAL  : usize = 1;
pub const STB_WEAK    : usize = 2;
pub const STB_LOPROC  : usize = 13;
pub const STB_HIPROC  : usize = 15;

pub const STT_NOTYPE  : u8 = 0;
pub const STT_OBJECT  : u8 = 1;
pub const STT_FUNC    : u8 = 2;
pub const STT_SECTION : u8 = 3;
pub const STT_FILE    : u8 = 4;
pub const STT_LOPROC  : u8 = 13;
pub const STT_HIPROC  : u8 = 15;

/** elf32 file header */
#[repr(C)]
pub struct Elf32Header {
    pub e_ident     : [u8; EI_NIDENT],
    pub e_type      : Elf32Half,
    pub e_machine   : Elf32Half,
    pub e_version   : Elf32Word,
    pub e_entry     : Elf32Addr,
    pub e_phoff     : Elf32Off,
    pub e_shoff     : Elf32Off,
    pub e_flags     : Elf32Word,
    pub e_ehsize    : Elf32Half,
    pub e_phentsize : Elf32Half,
    pub e_phnum     : Elf32Half,
    pub e_shentsize : Elf32Half,
    pub e_shnum     : Elf32Half,
    pub e_shstrndx  : Elf32Half,
}

/** elf32 section header */
#[repr(C)]
pub struct Elf32SectionHeader {
    pub sh_name       : Elf32Word,
    pub sh_type       : Elf32Word,
    pub sh_flags      : Elf32Word,
    pub sh_addr       : Elf32Addr,
    pub sh_offset     : Elf32Off,
    pub sh_size       : Elf32Word,
    pub sh_link       : Elf32Word,
    pub sh_info       : Elf32Word,
    pub sh_addralign  : Elf32Word,
    pub sh_entsize    : Elf32Word,
}

/** elf32 symbol */
#[repr(C)]
pub struct Elf32Symbol {
    pub st_name  : Elf32Word,
    pub st_value : Elf32Word,
    pub st_size  : Elf32Word,
    pub st_info  : u8,
    pub st_other : u8,
    pub st_shndx : Elf32Half,
}

/** elf32 program header */
#[repr(C)]
pub struct Elf32ProgramHeader {
    pub p_type   : Elf32Word,
    pub p_offset : Elf32Off,
    pub p_vaddr  : Elf32Addr,
    pub p_paddr  : Elf32Addr,
    pub p_filesz : Elf32Word,
    pub p_memsz  : Elf32Word,
    pub p_flags  : Elf32Word,
    pub p_align  : Elf32Word,
}

/** elf32 dynamic entry */
#[repr(C)]
pub struct Elf32Dynamic {
    pub d_tag : Elf32Sword,
    pub d_val : Elf32Addr,
}

unsafe fn binfmt_elf32_load(proc: *mut Process, vnode: *mut Node) -> isize {
    let mut err = 0;

    let vm_space = &mut (*proc).vm_space;
    let hdr: Elf32Header = core::mem::uninitialized();

    match (*vnode).read(0, core::mem::size_of_val(&hdr), &hdr as *const _ as *mut u8) {
        Ok(val) => {
            if val != core::mem::size_of_val(&hdr) {
                return -EINVAL;
            }
        },
        Err(err) => {
            return err.unwrap();
        }
    }

    let mut proc_heap = 0;
    let mut offset = hdr.e_phoff;
    
    for i in 0..hdr.e_phnum {
        let mut phdr: Elf32ProgramHeader = core::mem::uninitialized();
        
        match (*vnode).read(offset as usize, core::mem::size_of_val(&phdr), &phdr as *const _ as *mut u8) {
            Ok(val) => {
                if val != core::mem::size_of_val(&phdr) {
                    return -EINVAL;
                }
            },
            Err(err) => {
                return err.unwrap();
            }
        }

        if phdr.p_type as usize == PT_LOAD {
            let mut base   = phdr.p_vaddr;
            let mut filesz = phdr.p_filesz;
            let mut memsz  = phdr.p_memsz;
            let mut off    = phdr.p_offset;

            /* make sure vaddr is aligned */
            if base as usize & PAGE_MASK != 0 {
                memsz  += (base as usize & PAGE_MASK) as u32;
                filesz += (base as usize & PAGE_MASK) as u32;
                off    -= (base as usize & PAGE_MASK) as u32;
                base    = page_align!(base) as u32;
            }

            /* page align size */
            memsz = page_round!(memsz) as u32;

            let vm_entry = Box::leak(VmEntry::alloc(VmEntry::new()));

            vm_entry.base = base as usize;
            vm_entry.size = memsz as usize;
            vm_entry.off  = off as usize;

            /* access flags */
            vm_entry.flags |= if phdr.p_flags as usize & PF_R != 0 { VM_UR } else { 0 };
            vm_entry.flags |= if phdr.p_flags as usize & PF_W != 0 { VM_UW } else { 0 };
            vm_entry.flags |= if phdr.p_flags as usize & PF_X != 0 { VM_UX } else { 0 };

            /* TODO use W^X */

            vm_entry.qnode = (*vm_space).vm_entries.enqueue(vm_entry);

            if vm_entry.qnode.is_null() {
                return -ENOMEM;
            }

            vm_entry.vm_object = vm_object_vnode(vnode);

            if vm_entry.vm_object.is_null() {
                return -ENOMEM;
            }

            if vm_entry.flags & VM_UW != 0 {
                vm_entry.flags |= VM_COPY;
            }

            (*vm_entry.vm_object).incref();

            if base + memsz > proc_heap {
                proc_heap = base + memsz;
            }

            /* handle bss */
            if phdr.p_memsz != phdr.p_filesz {
                let bss = base + filesz;
                let bss_init_end = page_round!(base + filesz);

                if vm_entry.base + vm_entry.size > bss_init_end {
                    let sz = bss_init_end - vm_entry.base;
                    let split = Box::leak(VmEntry::alloc(VmEntry::new()));

                    split.base = bss_init_end;
                    split.size = vm_entry.size - sz;
                    split.flags = vm_entry.flags;
                    split.off = 0;

                    split.qnode = (*vm_space).vm_entries.enqueue(split);

                    vm_entry.size = sz;
                }

                /* fault in the page */
                core::ptr::write_bytes(bss as *mut u8, 0, bss_init_end - bss as usize);
                //memset((void *) bss, 0, bss_init_end-bss);
            }
        }

        offset += hdr.e_phentsize as u32;
    }

    (*proc).heap_start = proc_heap as usize;
    (*proc).heap       = proc_heap as usize;
    (*proc).entry      = hdr.e_entry as usize;

    return err;
}

pub unsafe fn binfmt_elf_check(vnode: *mut Node) -> isize {
    //print!("binfmt::elf::check(node={:?})\n", vnode);
    let mut hdr: Elf32Header = core::mem::uninitialized();
    (*vnode).read(0, core::mem::size_of_val(&hdr), &hdr as *const _ as *mut u8);

    /* Check header */
    if hdr.e_ident[EI_MAG0] == ELFMAG0 &&
       hdr.e_ident[EI_MAG1] == ELFMAG1 &&
       hdr.e_ident[EI_MAG2] == ELFMAG2 &&
       hdr.e_ident[EI_MAG3] == ELFMAG3 {
        return 0;
    }

    return -ENOEXEC;
}

pub unsafe fn binfmt_elf_load(proc: *mut Process, _path: *const u8, vnode: *mut Node) -> isize {
    //print!("binfmt::elf::load(proc={:?}, path={}, node={:?})\n", proc, cstr(_path), vnode);
    let mut err = 0;

    let mut hdr: Elf32Header = core::mem::uninitialized();

    match (*vnode).read(0, core::mem::size_of_val(&hdr), &hdr as *const _ as *mut u8) {
        Ok(val) => {
            if val != core::mem::size_of_val(&hdr) {
                return -EINVAL;
            }
        },
        Err(err) => {
            return err.unwrap();
        }
    }

    match hdr.e_ident[EI_CLASS] {
        ELFCLASS32 => {
            return binfmt_elf32_load(proc, vnode);
        },
        _ => {
            return -EINVAL;
        }
    }
}
