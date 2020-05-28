use prelude::*;

use crate::{print};

extern "C" {
    fn x86_lgdt(size: usize, addr: usize);
    fn x86_ltr(desc: usize);
}

#[repr(align(8))]
#[derive(Copy, Clone)]
pub struct GdtEntry(pub u64);

#[repr(C, align(8))]
pub struct TssEntry {
    link: u32,
    sp: usize,
    ss: u32,

    /* to know the actuall fields, consult intel manuals */
    _unused: [u32; 23],
}

pub static mut tss_entry: TssEntry = TssEntry { link: 0, sp: 0, ss: 0, _unused: [0; 23] };

/*
    uint32_t limit_lo : 16; /* Segment Limit 15:00 */
    uint32_t base_lo  : 16; /* Base Address 15:00 */

    uint32_t base_mid : 8;  /* Base Address 23:16 */
    uint32_t type     : 4;  /* Segment Type */
    uint32_t s        : 1;  /* Descriptor type (0=system, 1=code) */
    uint32_t dpl      : 2;  /* Descriptor Privellage Level */
    uint32_t p        : 1;  /* Segment present */

    uint32_t limit_hi : 4;  /* Segment Limit 19:16 */
    uint32_t avl      : 1;  /* Avilable for use by system software */
    uint32_t l        : 1;  /* Long mode segment (64-bit only) */
    uint32_t db       : 1;  /* Default operation size / upper Bound */
    uint32_t g        : 1;  /* Granularity */
    uint32_t base_hi  : 8;  /* Base Address 31:24 */




    uint32_t base_hi  : 8;  /* Base Address 31:24 */
    uint32_t g        : 1;  /* Granularity */
    uint32_t db       : 1;  /* Default operation size / upper Bound */
    uint32_t l        : 1;  /* Long mode segment (64-bit only) */
    uint32_t avl      : 1;  /* Avilable for use by system software */
    uint32_t limit_hi : 4;  /* Segment Limit 19:16 */

    uint32_t p        : 1;  /* Segment present */
    uint32_t dpl      : 2;  /* Descriptor Privellage Level */
    uint32_t s        : 1;  /* Descriptor type (0=system, 1=code) */
    uint32_t type     : 4;  /* Segment Type */
    uint32_t base_mid : 8;  /* Base Address 23:16 */

    uint32_t base_lo  : 16; /* Base Address 15:00 */
    uint32_t limit_lo : 16; /* Segment Limit 15:00 */
*/

impl GdtEntry {
    fn base_hi(mut self, a: u64) -> Self {
        let mask = 0xFF00000000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0xff) << 56)
            ;

        self
    }


    fn g(mut self, a: u64) -> Self {
        let mask = 0x0080000000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0x1) << 55)
            ;

        self
    }


    fn db(mut self, a: u64) -> Self {
        let mask = 0x0040000000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0x1) << 54)
            ;

        self
    }


    fn l(mut self, a: u64) -> Self {
        let mask = 0x0020000000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0x1) << 53)
            ;

        self
    }


    fn avl(mut self, a: u64) -> Self {
        let mask = 0x0010000000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0x1) << 52)
            ;

        self
    }


    fn limit_hi(mut self, a: u64) -> Self {
        let mask = 0x000F000000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0xf) << 48)
            ;

        self
    }


    fn p(mut self, a: u64) -> Self {
        let mask = 0x0000800000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0x1) << 47)
            ;

        self
    }


    fn dpl(mut self, a: u64) -> Self {
        let mask = 0x0000600000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0x3) << 45)
            ;

        self
    }


    fn s(mut self, a: u64) -> Self {
        let mask = 0x0000100000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0x1) << 44)
            ;

        self
    }


    fn _type(mut self, a: u64) -> Self {
        let mask = 0x00000F0000000000u64;

        self.0 = self.0 & !mask
            | ((a & 0xf) << 40)
            ;

        self
    }


    fn base_mid(mut self, a: u64) -> Self {
        let mask = 0x000000FF00000000u64;

        self.0 = self.0 & !mask
            | ((a & 0xff) << 32)
            ;

        self
    }


    fn base_lo(mut self, a: u64) -> Self {
        let mask = 0x00000000FFFF0000u64;

        self.0 = self.0 & !mask
            | ((a & 0xffff) << 16)
            ;

        self
    }


    fn limit_lo(mut self, a: u64) -> Self {
        let mask = 0x000000000000FFFFu64;

        self.0 = self.0 & !mask
            | ((a & 0xffff) << 0)
            ;

        self
    }
}

#[repr(C)]
pub enum SegmentType {
    ReadWriteData   = 0x2,
    ExecuteReadCode = 0xA,
    TaskState       = 0x9,
}

#[repr(C)]
pub enum PrivellageLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

#[repr(C)]
pub enum DescriptorType {
    System = 0,
    Code = 1,
}

#[repr(C)]
pub enum Granularity {
    Byte = 0,
    Page = 1,
}

#[repr(C)]
pub enum OperationSize {
    Bit16 = 0,
    Bit32 = 1,
}

impl GdtEntry {
    pub fn new() -> Self {
        GdtEntry(0)
    }

    pub fn base(mut self, addr: u64) -> Self {
        self.base_hi((addr >> 24) & 0xFF)
            .base_mid((addr >> 16) & 0xFF)
            .base_lo((addr >> 0) & 0xFFFF)
    }

    pub fn limit(mut self, addr: u64) -> Self {
        self.limit_hi((addr >> 16) & 0xF)
            .limit_lo((addr >> 0)  & 0xFFFF)
    }

    pub fn granularity(mut self, gran: Granularity) -> Self {
        self.g(gran as u64)
    }

    pub fn operation_size(mut self, db: OperationSize) -> Self {
        self.db(db as u64)
    }

    pub fn upper_bound(mut self, db: OperationSize) -> Self {
        self.operation_size(db)
    }

    pub fn available(mut self, avl: bool) -> Self {
        self.avl(avl as u64)
    }

    pub fn present(mut self, p: bool) -> Self {
        self.p(p as u64)
    }

    pub fn privellage_level(mut self, dpl: PrivellageLevel) -> Self {
        self.dpl(dpl as u64)
    }

    pub fn descriptor_type(mut self, s: DescriptorType) -> Self {
        self.s(s as u64)
    }

    pub fn segment_type(mut self, _type: SegmentType) -> Self {
        self._type(_type as u64)
    }
}

#[no_mangle]
pub static mut gdt: [GdtEntry; 256] = [GdtEntry(0); 256];

pub unsafe fn x86_gdt_setup() {
    let base = 0;
    let limit = 0xFFFF_FFFF;

    //gdt[0] = GdtEntry(0x0000000000000000u64);
    //gdt[1] = GdtEntry(0x00CF9A000000FFFFu64);
    //gdt[2] = GdtEntry(0x00CF92000000FFFFu64);
    //gdt[3] = GdtEntry(0x00CFFA000000FFFFu64);
    //gdt[4] = GdtEntry(0x00CFF2000000FFFFu64);

    /* null segment */
    gdt[0] = GdtEntry::new();

    /* code segment - kernel */
    gdt[1] = GdtEntry::new()
        .base(base)
        .limit(limit)
        .segment_type(SegmentType::ExecuteReadCode)
        .descriptor_type(DescriptorType::Code)
        .privellage_level(PrivellageLevel::Ring0)
        .operation_size(OperationSize::Bit32)
        .granularity(Granularity::Page)
        .available(false)
        .present(true);

    /* data segment - kernel */
    gdt[2] = GdtEntry::new()
        .base(base)
        .limit(limit)
        .segment_type(SegmentType::ReadWriteData)
        .descriptor_type(DescriptorType::Code)
        .privellage_level(PrivellageLevel::Ring0)
        .operation_size(OperationSize::Bit32)
        .granularity(Granularity::Page)
        .available(false)
        .present(true);

    /* code segment - user */
    gdt[3] = GdtEntry::new()
        .base(base)
        .limit(limit)
        .segment_type(SegmentType::ExecuteReadCode)
        .descriptor_type(DescriptorType::Code)
        .privellage_level(PrivellageLevel::Ring3)
        .operation_size(OperationSize::Bit32)
        .granularity(Granularity::Page)
        .available(false)
        .present(true);

    /* data segment - user */
    gdt[4] = GdtEntry::new()
        .base(base)
        .limit(limit)
        .segment_type(SegmentType::ReadWriteData)
        .descriptor_type(DescriptorType::Code)
        .privellage_level(PrivellageLevel::Ring3)
        .operation_size(OperationSize::Bit32)
        .granularity(Granularity::Page)
        .available(false)
        .present(true);

    x86_lgdt(core::mem::size_of_val(&gdt) - 1, &gdt as *const _ as usize);
}

pub unsafe fn x86_tss_setup(sp: usize) {
    tss_entry.ss = 0x10;
    tss_entry.sp = sp;

    let tss_base  = &tss_entry as *const _ as usize;
    let tss_limit = core::mem::size_of_val(&tss_entry) - 1;

    /* TSS Segment */
    gdt[5] = GdtEntry::new()
        .base(tss_base as u64)
        .limit(tss_limit as u64)
        .segment_type(SegmentType::TaskState)
        .descriptor_type(DescriptorType::System)
        .privellage_level(PrivellageLevel::Ring3)
        .present(true)
        .available(false)
        .operation_size(OperationSize::Bit16)
        .granularity(Granularity::Byte);

    x86_ltr(0x28 | (PrivellageLevel::Ring3 as usize));
}

pub unsafe fn x86_kernel_stack_set(sp: usize) {
    tss_entry.sp = sp;
}
