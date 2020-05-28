use prelude::*;

use arch::i386::cpu::idt::x86_idt_gate_set;
use crate::arch::i386::include::cpu::io::IOAddr;
use crate::arch::i386::include::cpu::cpu::X86Regs;
use crate::{print};

pub const PIC_CMD:  usize = 0x00;
pub const PIC_DATA: usize = 0x01;

static mut master: IOAddr = IOAddr::empty();
static mut slave:  IOAddr = IOAddr::empty();

/*
 * ```
 * ICW1 (Sent on COMMAND port of each PIC)
 *
 * | A7 | A6 | A5 | 1 | LTIM | ADI | SINGL | IC4 |
 *   |____|____|          |     |     |       |______ 1=ICW4 REQUIRED
 *        |               |     |   1=SINGEL          0=ICW4 NOT REQUIRED
 *  A7:5 OF INTERRUPT     |     |   0=CASCADED
 *   VECTOR ADDRESS       |     |
 * (MCS-80/85 MODE ONLY!) | CALL ADDRESS INTERVAL (IGNORED IN 8086 MODE)
 *                        | 1=INTERVAL OF 4
 *                        | 0=INTERVAL OF 8
 *                        |
 *              1=LEVEL TRIGERED MODE
 *              0= EDGE TRIGERED MODE
 *
 *******************************************************************************
 * ICW2 (Sent on DATA port of each PIC)
 *
 * | A15 | A14 | A13 | A12 | A11 | A10 | A9 | A8 |
 *   |_____|_____|_____|_____|_____|_____|____|
 *                        |
 *     A15:8 OF INTERRUPT VECTOR ADDRESS (MCS-80/85 MODE)
 *     T7:3  OF INTERRUPT VECTOR ADDRESS (8086 MODE)
 *
 *******************************************************************************
 * ICW3 (Sent on DATA port of each PIC)
 *
 * --FOR MASTER:
 * | S7 | S6 | S5 | S4 | S3 | S2 | S1 | S0 |
 *   |____|____|____|____|____|____|____|
 *                  |
 *       1=IR LINE HAS SLAVE (CASCADED)
 *       0=IR LINE DOES NOT HAVE SLAVE (SINGLE)
 *
 * --FOR SLAVE:
 * | 0 | 0 | 0 | 0 | 0 | ID2 | ID1 | ID0 |
 *                       |_____|_____|
 *                             |
 *                         SLAVE ID
 *
 *******************************************************************************
 * ICW4 (Sent on DATA port of each PIC)
 * Well, I am too lazy to write this one XD so I will just tell you that setting
 * the least-significant bit sets the PIC to 8086 MODE
 *```
 */

/* both master and slave use the same ICW1 */
const ICW1: u8 = 0x11;

/* interrupts (from master) start from offset 32 */
const ICW2_MASTER: u8 = 0x20;

/* interrupts (from slave)  start from offset 40 */
const ICW2_SLAVE: u8 = 0x28;

/* master has a slave attached to IR 2 */
const ICW3_MASTER: u8 = 0x04;

/* slave id is 2 */
const ICW3_SLAVE: u8 = 0x02;

/* sets pic to 8086 mode */
const ICW4: u8 = 0x01;

/* The mask value currently on slave:master */
static mut pic_mask: u16 = 0xFFFF;

pub unsafe fn x86_irq_mask(irq: usize) {
    if (irq < 8) {  /* Master */
        pic_mask |= 1 << irq;
        master.out8(PIC_DATA, (pic_mask & 0xFF) as u8);
    } else if (irq < 16) {  /* Slave */
        pic_mask |= 1 << irq;
        slave.out8(PIC_DATA, ((pic_mask >> 8) & 0xFF) as u8);
    } else {
        panic!("Invalid IRQ number");
    }
}

pub unsafe fn x86_irq_unmask(irq: usize) {
    if (irq < 8) {  /* Master */
        pic_mask &= !(1 << irq);
        master.out8(PIC_DATA, (pic_mask & 0xFF) as u8);
    } else if (irq < 16) {  /* Slave */
        pic_mask &= !(1 << irq);
        pic_mask &= !(1 << 2);  /* Unmask slave */
        slave.out8(PIC_DATA, ((pic_mask >> 8) & 0xFF) as u8);
    } else {
        panic!("Invalid IRQ number");
    }
}

unsafe fn x86_irq_remap() {
    /*
     * Initializes PIC & remaps PIC interrupts to different interrupt
     * numbers so as not to conflict with CPU exceptions
     */

    master.out8(PIC_CMD,  ICW1);
    slave.out8(PIC_CMD,  ICW1);
    master.out8(PIC_DATA, ICW2_MASTER);
    slave.out8(PIC_DATA, ICW2_SLAVE);
    master.out8(PIC_DATA, ICW3_MASTER);
    slave.out8(PIC_DATA, ICW3_SLAVE);
    master.out8(PIC_DATA, ICW4);
    slave.out8(PIC_DATA, ICW4);
}

extern "C" {
    fn __x86_irq0();
    fn __x86_irq1();
    fn __x86_irq2();
    fn __x86_irq3();
    fn __x86_irq4();
    fn __x86_irq5();
    fn __x86_irq6();
    fn __x86_irq7();
    fn __x86_irq8();
    fn __x86_irq9();
    fn __x86_irq10();
    fn __x86_irq11();
    fn __x86_irq12();
    fn __x86_irq13();
    fn __x86_irq14();
    fn __x86_irq15();

    static __x86_isr_int_num: u32;
}

static mut irq_handlers: [Option<unsafe fn(_: *const X86Regs)>; 16] = [None; 16];

pub unsafe fn x86_irq_handler_install(irq: usize, handler: unsafe fn(_: *const X86Regs)) {
    if (irq < 16) {
        x86_irq_unmask(irq);
        irq_handlers[irq] = Some(handler);
    }
}

pub unsafe fn x86_irq_handler_uninstall(irq: usize) {
    if (irq < 16) {
        x86_irq_mask(irq);
        irq_handlers[irq] = None;
    }
}

const IRQ_ACK: u8 = 0x20;
unsafe fn x86_irq_ack(irq: usize) {
    if irq > 7 {
        /* IRQ fired from the Slave PIC */
        slave.out8(PIC_CMD, IRQ_ACK);
    }

    master.out8(PIC_CMD, IRQ_ACK);
}

#[no_mangle]
pub unsafe extern "C" fn __x86_irq_handler(r: *const X86Regs) {

    let mut handler;

    if (__x86_isr_int_num > 47 || __x86_isr_int_num < 32) {
        /* Out of range */
        handler = None;
    } else {
        handler = irq_handlers[(__x86_isr_int_num - 32) as usize];
    }

    x86_irq_ack((__x86_isr_int_num - 32) as usize);

    if (handler.is_some()) {
        (handler.unwrap())(r);
    }
}

unsafe fn x86_irq_gates_setup() {
    x86_idt_gate_set(32, __x86_irq0 as *const u8 as usize);
    x86_idt_gate_set(33, __x86_irq1 as *const u8 as usize);
    x86_idt_gate_set(34, __x86_irq2 as *const u8 as usize);
    x86_idt_gate_set(35, __x86_irq3 as *const u8 as usize);
    x86_idt_gate_set(36, __x86_irq4 as *const u8 as usize);
    x86_idt_gate_set(37, __x86_irq5 as *const u8 as usize);
    x86_idt_gate_set(38, __x86_irq6 as *const u8 as usize);
    x86_idt_gate_set(39, __x86_irq7 as *const u8 as usize);
    x86_idt_gate_set(40, __x86_irq8 as *const u8 as usize);
    x86_idt_gate_set(41, __x86_irq9 as *const u8 as usize);
    x86_idt_gate_set(42, __x86_irq10 as *const u8 as usize);
    x86_idt_gate_set(43, __x86_irq11 as *const u8 as usize);
    x86_idt_gate_set(44, __x86_irq12 as *const u8 as usize);
    x86_idt_gate_set(45, __x86_irq13 as *const u8 as usize);
    x86_idt_gate_set(46, __x86_irq14 as *const u8 as usize);
    x86_idt_gate_set(47, __x86_irq15 as *const u8 as usize);
}

unsafe fn x86_pic_probe() -> isize {
    /* mask all slave irqs */
    slave.out8(PIC_DATA, 0xFF);

    /* mask all master irqs -- except slave cascade */
    master.out8(PIC_DATA, 0xDF);

    /* check if there is a devices listening to port */
    if master.in8(PIC_DATA) != 0xDF {
        return -1;
    }

    return 0;
}

pub unsafe fn x86_pic_disable() {
    /* done by masking all irqs */
    slave.out8(PIC_DATA, 0xFF);
    master.out8(PIC_DATA, 0xFF);
}

#[no_mangle]
pub unsafe fn x86_pic_setup(_master: *const IOAddr, _slave: *const IOAddr) -> isize {
    master = *_master;
    slave  = *_slave;

    if (x86_pic_probe() != 0) {
        print!("i8259: controller not found\n");
        return -1;
    }

    print!("i8259: initializing [master: {:p} ({}), salve: {:p} ({})]\n",
            master.addr as *const u8, master.type_str(),
            slave.addr  as *const u8, slave.type_str());

    /* initialize */
    x86_irq_remap();

    /* mask all interrupts */
    x86_pic_disable();

    /* setup call gates */
    x86_irq_gates_setup();
    return 0;
}

