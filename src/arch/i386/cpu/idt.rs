use prelude::*;

#[repr(packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    pub offset_lo : u16,
    pub selector  : u16,
    pub _unused   : u8,
    pub flags     : u8,
    pub offset_hi : u16,
}

impl IdtEntry {
    const fn empty() -> Self {
        IdtEntry {
            offset_lo: 0,
            selector: 0,
            _unused: 0,
            flags: 0,
            offset_hi: 0,
        }
    }
}

#[repr(packed)]
#[derive(Copy, Clone)]
struct IdtPointer {
    pub limit: u16,
    pub base: usize,
}

impl IdtPointer {
    const fn empty() -> Self {
        IdtPointer {
            limit: 0,
            base: 0,
        }
    }
}

extern "C" {
    fn x86_lidt(_: usize);
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];
static mut IDT_POINTER: IdtPointer = IdtPointer::empty();

/* sets interrupt gates in kernel code segment */
pub unsafe fn x86_idt_gate_set(id: usize, offset: usize) {
    IDT[id].offset_lo  = ((offset >> 0x00) & 0xFFFF) as u16;
    IDT[id].offset_hi  = ((offset >> 0x10) & 0xFFFF) as u16;

    IDT[id].selector   = 0x8;
    IDT[id].flags      = 0x8E;
}

/* sets interrupt gates in user code segment */
pub unsafe fn x86_idt_gate_user_set(id: usize, offset: usize) {
    IDT[id].offset_lo  = ((offset >> 0x00) & 0xFFFF) as u16;
    IDT[id].offset_hi  = ((offset >> 0x10) & 0xFFFF) as u16;

    IDT[id].selector   = 0x8;
    IDT[id].flags      = 0xEE;
}

pub unsafe fn x86_idt_setup() {
    IDT_POINTER.limit = (core::mem::size_of_val(&IDT) - 1) as u16;
    IDT_POINTER.base  = &IDT as *const _ as usize;
    x86_lidt(&IDT_POINTER as *const _ as usize);
}
