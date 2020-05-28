use prelude::*;

use crate::arch::i386::include::cpu::cpu::X86Regs;
use crate::arch::i386::platform::misc::cmos::x86_cmos_setup;
use crate::arch::i386::platform::misc::pit::x86_pit_setup;
use crate::arch::i386::platform::misc::pit::x86_pit_period_set;
use crate::arch::i386::platform::misc::pic::x86_pic_setup;
use crate::arch::i386::platform::misc::pic::x86_irq_handler_install;
use crate::arch::i386::include::cpu::io::*;
use crate::{print};

/* PCI bus */
const PCI_ADDR: usize = 0xCF8;
const PCI_TYPE: u8 = IOADDR_PORT;

/* i8042 PS/2 controller */
const I8042_ADDR: usize = 0x60;
const I8042_TYPE: u8 = IOADDR_PORT;

/* i8254 PIT controller */
const I8254_ADDR: usize = 0x40;
const I8254_TYPE: u8 = IOADDR_PORT;

const PIC_MASTER: usize = 0x20;
const PIC_SLAVE:  usize = 0xA0;

unsafe fn x86_pc_pic_init() -> isize {
    let mut pic_master: IOAddr = IOAddr {
        addr: PIC_MASTER,
        _type: IOADDR_PORT,
    };

    let mut pic_slave: IOAddr = IOAddr {
        addr: PIC_SLAVE,
        _type: IOADDR_PORT,
    };

    return x86_pic_setup(&mut pic_master, &mut pic_slave);
}

unsafe fn x86_pc_pci_init() {
    print!("x86: initializing pci\n");

    let pci: IOAddr = IOAddr {
        addr: PCI_ADDR,
        _type: PCI_TYPE,
    };

    //pci_ioaddr_set(&pci);
}

unsafe fn x86_pc_i8042_init() -> isize {
    let i8042 = IOAddr {
        addr: I8042_ADDR,
        _type: I8042_TYPE,
    };

    //return x86_i8042_setup(&i8042);
    return -1;
}

unsafe fn x86_pc_pit_init() -> isize {
    let mut pit = IOAddr {
        addr: I8254_ADDR,
        _type: I8254_TYPE,
    };

    return x86_pit_setup(&mut pit);
}

unsafe fn x86_pc_cmos_init() -> isize {
    let mut cmos = IOAddr {
        addr: 0x70,
        _type: IOADDR_PORT,
    };

    return x86_cmos_setup(&mut cmos);
}

const PIT_IRQ: usize = 0;

pub unsafe fn platform_timer_setup(period_ns: u32, handler: unsafe fn(_: *const X86Regs)) -> u32 {
    let period = x86_pit_period_set(period_ns);
    x86_irq_handler_install(PIT_IRQ, handler);
    return period;
}

pub unsafe fn platform_init() -> isize {
    x86_pc_pci_init();
    x86_pc_pic_init();
    x86_pc_i8042_init();
    x86_pc_pit_init();
    x86_pc_cmos_init();
    
    return 0;
}

