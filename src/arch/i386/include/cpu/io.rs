use prelude::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IOAddr {
    pub _type: u8,
    pub addr: usize,
}

#[inline]
unsafe fn __inb(port: usize) -> u8 {
    let ret;
    llvm_asm!("inb %dx, %al":"={al}"(ret):"{dx}"(port));
    ret
}

#[inline]
unsafe fn __inw(port: usize) -> u16 {
    let ret;
    llvm_asm!("inw %dx, %ax":"={ax}"(ret):"{dx}"(port));
    ret
}

#[inline]
unsafe fn __inl(port: usize) -> u32 {
    let ret;
    llvm_asm!("inl %dx, %eax":"={eax}"(ret):"{dx}"(port));
    ret
}

#[inline]
unsafe fn __outb(port: usize, value: u8) {
    //llvm_asm!("outb %al, %dx"::"d"((port)),"a"((value)));
    llvm_asm!("outb %al, %dx"::"{edx}"((port)),"{al}"((value)));
}

#[inline]
unsafe fn __outw(port: usize, value: u16) {
    llvm_asm!("outw %ax, %dx"::"{dx}"(port),"{ax}"(value));
}

#[inline]
unsafe fn __outl(port: usize, value: u32) {
    llvm_asm!("outl %eax, %dx"::"{dx}"(port),"{eax}"(value));
}

/*
#define __io_wait() \
({ \
    llvm_asm volatile ( "jmp 1f\n\t" \
                   "1:jmp 2f\n\t" \
                   "2:" ); \
})
*/

#[inline]
unsafe fn __mmio_inb(addr: usize) -> u8 {
    core::ptr::read_volatile(addr as *const u8)
}

#[inline]
unsafe fn __mmio_inw(addr: usize) -> u16 {
    core::ptr::read_volatile(addr as *const u16)
}

#[inline]
unsafe fn __mmio_inl(addr: usize) -> u32 {
    core::ptr::read_volatile(addr as *const u32)
}

#[inline]
unsafe fn __mmio_inq(addr: usize) -> u64 {
    core::ptr::read_volatile(addr as *const u64)
}

#[inline]
unsafe fn __mmio_outb(addr: usize, v: u8) {
    core::ptr::write_volatile(addr as *mut u8, v);
}

#[inline]
unsafe fn __mmio_outw(addr: usize, v: u16) {
    core::ptr::write_volatile(addr as *mut u16, v);
}

#[inline]
unsafe fn __mmio_outl(addr: usize, v: u32) {
    core::ptr::write_volatile(addr as *mut u32, v);
}

#[inline]
unsafe fn __mmio_outq(addr: usize, v: u64) {
    core::ptr::write_volatile(addr as *mut u64, v);
}

pub const IOADDR_PORT:   u8 = 1;
pub const IOADDR_MMIO8:  u8 = 2;
pub const IOADDR_MMIO16: u8 = 3;
pub const IOADDR_MMIO32: u8 = 4;

impl IOAddr {
    pub const fn empty() -> Self {
        IOAddr {
            _type: 0,
            addr: 0,
        }
    }

    #[inline]
    pub fn type_str(&self) -> &str {
        match self._type {
            IOADDR_PORT   => "pio",
            IOADDR_MMIO8  => "mmio8",
            IOADDR_MMIO16 => "mmio16",
            IOADDR_MMIO32 => "mmio32",
            _ => ""
        }
    }

    #[inline]
    pub unsafe fn in8(&self, off: usize) -> u8 {
        match self._type {
            IOADDR_PORT   => __inb(self.addr + off),
            IOADDR_MMIO8  => __mmio_inb(self.addr + off),
            IOADDR_MMIO16 => __mmio_inb(self.addr + (off << 1)),
            IOADDR_MMIO32 => __mmio_inb(self.addr + (off << 2)),
            _ => 0
        }
    }

    #[inline]
    pub unsafe fn in16(&self, off: usize) -> u16 {
        match self._type {
            IOADDR_PORT   => __inw(self.addr + off),
            IOADDR_MMIO8  => __mmio_inw(self.addr + off),
            IOADDR_MMIO16 => __mmio_inw(self.addr + (off << 1)),
            IOADDR_MMIO32 => __mmio_inw(self.addr + (off << 2)),
            _ => 0,
        }
    }

    #[inline]
    pub unsafe fn in32(&self, off: usize) -> u32 {
        match self._type {
            IOADDR_PORT   => __inl(self.addr + off),
            IOADDR_MMIO8  => __mmio_inl(self.addr + off),
            IOADDR_MMIO16 => __mmio_inl(self.addr + (off << 1)),
            IOADDR_MMIO32 => __mmio_inl(self.addr + (off << 2)),
            _ => 0
        }
    }

    #[inline]
    pub unsafe fn out8(&self, off: usize, val: u8) {
        match self._type {
            IOADDR_PORT   => __outb(self.addr + off, val),
            IOADDR_MMIO8  => __mmio_outb(self.addr + off, val),
            IOADDR_MMIO16 => __mmio_outb(self.addr + (off << 1), val),
            IOADDR_MMIO32 => __mmio_outb(self.addr + (off << 2), val),
            _ => {}
        }
    }

    #[inline]
    pub unsafe fn out16(&self, off: usize, val: u16) {
        match self._type {
            IOADDR_PORT   => __outw(self.addr + off, val),
            IOADDR_MMIO8  => __mmio_outw(self.addr + off, val),
            IOADDR_MMIO16 => __mmio_outw(self.addr + (off << 1), val),
            IOADDR_MMIO32 => __mmio_outw(self.addr + (off << 2), val),
            _ => {}
        }
    }

    #[inline]
    pub unsafe fn out32(&self, off: usize, val: u32) {
        match self._type {
            IOADDR_PORT   => __outl(self.addr + off, val),
            IOADDR_MMIO8  => __mmio_outl(self.addr + off, val),
            IOADDR_MMIO16 => __mmio_outl(self.addr + (off << 1), val),
            IOADDR_MMIO32 => __mmio_outl(self.addr + (off << 2), val),
            _ => {}
        }
    }
}
