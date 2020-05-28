use prelude::*;

/* how many bytes from the start of the file we search for the header. */
pub const MULTIBOOT_SEARCH: u32 = 8192;

/* the magic field should contain this. */
pub const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BADB002;

/* this should be in %eax. */
pub const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;

/* the bits in the required part of flags field we don't support. */
pub const MULTIBOOT_UNSUPPORTED: u32 = 0x0000fffc;

/* alignment of multiboot modules. */
pub const MULTIBOOT_MOD_ALIGN: u32 = 0x00001000;

/* alignment of the multiboot info structure. */
pub const MULTIBOOT_INFO_ALIGN: u32 = 0x00000004;

/* flags set in the 'flags' member of the multiboot header. */

/* align all boot modules on i386 page (4KB) boundaries. */
pub const MULTIBOOT_PAGE_ALIGN:  u32 = 0x00000001;

/* must pass memory information to OS. */
pub const MULTIBOOT_MEMORY_INFO: u32 = 0x00000002;

/* must pass video information to OS. */
pub const MULTIBOOT_VIDEO_MODE:  u32 = 0x00000004;

/* this flag indicates the use of the address fields in the header. */
pub const MULTIBOOT_AOUT_KLUDGE: u32 = 0x00010000;

/* flags to be set in the 'flags' member of the multiboot info structure. */

/* is there basic lower/upper memory information? */
pub const MULTIBOOT_INFO_MEMORY:  u32 = 0x00000001;
/* is there a boot device set? */
pub const MULTIBOOT_INFO_BOOTDEV: u32 = 0x00000002;
/* is the command-line defined? */
pub const MULTIBOOT_INFO_CMDLINE: u32 = 0x00000004;
/* are there modules to do something with? */
pub const MULTIBOOT_INFO_MODS:    u32 = 0x00000008;

/* these next two are mutually exclusive */

/* is there a symbol table loaded? */
pub const MULTIBOOT_INFO_AOUT_SYMS:        u32 = 0x00000010;
/* is there an ELF section header table? */
pub const MULTIBOOT_INFO_ELF_SHDR:         u32 = 0x00000020;

/* is there a full memory map? */
pub const MULTIBOOT_INFO_MEM_MAP:          u32 = 0x00000040;

/* is there drive info? */
pub const MULTIBOOT_INFO_DRIVE_INFO:       u32 = 0x00000080;

/* is there a config table? */
pub const MULTIBOOT_INFO_CONFIG_TABLE:     u32 = 0x00000100;

/* is there a boot loader name? */
pub const MULTIBOOT_INFO_BOOT_LOADER_NAME: u32 = 0x00000200;

/* is there a APM table? */
pub const MULTIBOOT_INFO_APM_TABLE:        u32 = 0x00000400;

/* is there video information? */
pub const MULTIBOOT_INFO_VIDEO_INFO:       u32 = 0x00000800;

#[repr(C)]
pub struct MultibootHeader {
    /* must be MULTIBOOT_MAGIC - see above. */
    pub magic: u32,

    /* feature flags. */
    pub flags: u32,

    /* the above fields plus this one must equal 0 mod 2^32. */
    pub checksum: u32,

    /* these are only valid if MULTIBOOT_AOUT_KLUDGE is set. */
    pub header_addr: u32,
    pub load_addr: u32,
    pub load_end_addr: u32,
    pub bss_end_addr: u32,
    pub entry_addr: u32,

    /* these are only valid if MULTIBOOT_VIDEO_MODE is set. */
    pub mode_type: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

/* the section header table for ELF. */
#[repr(C)]
pub struct MultibootElfSectionHeaderTable {
  pub num: u32,
  pub size: u32,
  pub addr: u32,
  pub shndx: u32,
}

#[repr(C)]
pub struct MultibootInfo {
  /* multiboot info version number */
  pub flags: u32,

  /* available memory from BIOS */
  pub mem_lower: u32,
  pub mem_upper: u32,

  /* "root" partition */
  pub boot_device: u32,

  /* kernel command line */
  pub cmdline: u32,

  /* boot-module list */
  pub mods_count: u32,
  pub mods_addr: u32,

  pub elf_sec: MultibootElfSectionHeaderTable,

  /* memory mapping buffer */
  pub mmap_length: u32,
  pub mmap_addr: u32,

  /* drive info buffer */
  pub drives_length: u32,
  pub drives_addr: u32,

  /* ROM configuration table */
  pub config_table: u32,

  /* boot loader name */
  pub boot_loader_name: u32,

  /* APM table */
  pub apm_table: u32,

  /* video */
  pub vbe_control_info: u32,
  pub vbe_mode_info: u32,
  pub vbe_mode: u16,
  pub vbe_interface_seg: u16,
  pub vbe_interface_off: u16,
  pub vbe_interface_len: u16,
}

pub const MULTIBOOT_MEMORY_AVAILABLE: u32 = 1;
pub const MULTIBOOT_MEMORY_RESERVED:  u32 = 2;

#[repr(packed)]
pub struct MultibootMmapEntry {
    pub size: u32,
    pub addr: u64,
    pub len:  u64,
    pub map_type: u32,
}

#[repr(C)]
pub struct MultibootModList {
    /* the memory used goes from bytes 'mod_start' to 'mod_end-1' inclusive */
    pub mod_start: u32,
    pub mod_end: u32,

    /* module command line */
    pub cmdline: u32,

    /* padding to take it to 16 bytes (must be zero) */
    pub pad: u32,
}

extern "C" {
    pub static multiboot_signature: u32;
    pub static multiboot_info: u32;
}
