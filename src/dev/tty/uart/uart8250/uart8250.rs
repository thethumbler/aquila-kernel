use prelude::*;

use crate::dev::tty::uart::uart::Uart;
use crate::dev::tty::uart::uart::uart_register;
use crate::dev::tty::uart::uart::uart_transmit_handler;
use crate::dev::tty::uart::uart::uart_recieve_handler;
use crate::arch::i386::include::cpu::io::IOAddr;
use crate::arch::i386::include::cpu::io::IOADDR_PORT;
use crate::arch::i386::platform::misc::pic::x86_irq_handler_install;
use crate::arch::i386::include::cpu::cpu::X86Regs;

const UART_IER: usize = 1;
const UART_FCR: usize = 2;
const UART_LCR: usize = 3;
const UART_MCR: usize = 4;
const UART_DLL: usize = 0;
const UART_DLH: usize = 1;
const UART_LCR_DLAB: u8 = 0x80;

static mut io8250: IOAddr = IOAddr {
    addr:  0x3F8,
    _type: IOADDR_PORT,
};

const UART_8250_IRQ: usize = 4;


unsafe fn serial_empty() -> bool {
    return io8250.in8(5) & 0x20 != 0;
}

unsafe fn serial_received() -> bool {
   return io8250.in8(5) & 0x01 != 0;
}

unsafe fn uart_8250_receive(_u: *mut Uart) -> u8 {
    return io8250.in8(0);
}

unsafe fn uart_8250_transmit(_u: *mut Uart, c: u8) -> isize {
    io8250.out8(0, c);
    return 1;
}

unsafe fn uart_8250_irq(_: *const X86Regs) {
    if (serial_received()) {
        if !uart_8250.vnode.is_null() {
            /* if open */
            uart_recieve_handler(&mut uart_8250, 1);
        }
    }

    if (serial_empty()) {
        if !uart_8250.vnode.is_null() {
            /* if open */
            uart_transmit_handler(&mut uart_8250, 1);
        }
    }
}

unsafe fn uart_8250_comm_init(_u: *mut Uart) {
    /* flush all output before reseting */
    while (!serial_empty()) {}

    /* disable all interrupts */
    io8250.out8(UART_IER, 0x00);

    /* 8 bits, no parity, one stop bit */
    io8250.out8(UART_LCR, 0x03);
    /* enalbe fifo, clear, 14 byte threshold */
    io8250.out8(UART_FCR, 0xC7);
    /* DTR + RTS */
    io8250.out8(UART_MCR, 0x0B);

    /* enable DLAB and set divisor */
    let lcr = io8250.in8(UART_LCR);
    /* enable DLAB */
    io8250.out8(UART_LCR, lcr | UART_LCR_DLAB);
    /* set divisor to 3 */
    io8250.out8(UART_DLL, 0x03);
    io8250.out8(UART_DLH, 0x00);
    io8250.out8(UART_LCR, lcr & !UART_LCR_DLAB);

    /* enable data/empty interrupt */
    io8250.out8(UART_IER, 0x01);
}

unsafe fn uart_8250_init() -> isize {
    //serial_init();
    x86_irq_handler_install(UART_8250_IRQ, uart_8250_irq);
    uart_register(0, &mut uart_8250);
    return 0;
}

static mut uart_8250: Uart = Uart {
    name:     b"8250\0".as_ptr(),
    init:     Some(uart_8250_comm_init),
    transmit: Some(uart_8250_transmit),
    receive:  Some(uart_8250_receive),
    tty:      core::ptr::null_mut(),
    _in:      core::ptr::null_mut(),
    _out:     core::ptr::null_mut(),
    vnode:    core::ptr::null_mut(),
};

module_init!(uart_8250, Some(uart_8250_init), None);
