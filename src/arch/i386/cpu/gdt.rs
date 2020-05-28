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

pub static mut TSS_ENTRY: TssEntry = TssEntry { link: 0, sp: 0, ss: 0, _unused: [0; 23] };

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

static mut GDT: [GdtEntry; 256] = [GdtEntry(0); 256];

pub unsafe fn x86_gdt_setup() {
    let base = 0;
    let limit = 0xFFFF_FFFF;

    /* null segment */
    GDT[0] = GdtEntry::new();

    /* code segment - kernel */
    GDT[1] = GdtEntry::new()
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
    GDT[2] = GdtEntry::new()
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
    GDT[3] = GdtEntry::new()
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
    GDT[4] = GdtEntry::new()
        .base(base)
        .limit(limit)
        .segment_type(SegmentType::ReadWriteData)
        .descriptor_type(DescriptorType::Code)
        .privellage_level(PrivellageLevel::Ring3)
        .operation_size(OperationSize::Bit32)
        .granularity(Granularity::Page)
        .available(false)
        .present(true);

    x86_lgdt(core::mem::size_of_val(&GDT) - 1, &GDT as *const _ as usize);
}

pub unsafe fn x86_tss_setup(sp: usize) {
    TSS_ENTRY.ss = 0x10;
    TSS_ENTRY.sp = sp;

    let tss_base  = &TSS_ENTRY as *const _ as usize;
    let tss_limit = core::mem::size_of_val(&TSS_ENTRY) - 1;

    /* TSS Segment */
    GDT[5] = GdtEntry::new()
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
    TSS_ENTRY.sp = sp;
}
