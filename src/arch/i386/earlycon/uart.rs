use prelude::*;

use crate::arch::earlycon::earlycon::EarlyConsole;
use crate::arch::include::cpu::io::IOAddr;
use crate::arch::include::cpu::io::IOADDR_PORT;

static EARLYCON_UART_IOADDR: IOAddr = IOAddr {
    addr: 0x3F8,
    _type: IOADDR_PORT,
};

const UART_IER: usize = 1;
const UART_FCR: usize = 2;
const UART_LCR: usize = 3;
const UART_MCR: usize = 4;
const UART_DLL: usize = 0;
const UART_DLH: usize = 1;
const UART_LCR_DLAB: u8 = 0x80;

unsafe fn serial_init() {
    /* disable all interrupts */
    EARLYCON_UART_IOADDR.out8(UART_IER, 0x00);
    /* disable fifo */
    EARLYCON_UART_IOADDR.out8(UART_FCR, 0x00);
    /* 8 bits, no parity, one stop bit */
    EARLYCON_UART_IOADDR.out8(UART_LCR, 0x03);
    /* RTS + DTR */
    EARLYCON_UART_IOADDR.out8(UART_MCR, 0x03);

    let lcr = EARLYCON_UART_IOADDR.in8(UART_LCR);

    /* enable DLAB */
    EARLYCON_UART_IOADDR.out8(UART_LCR, lcr | UART_LCR_DLAB);
    /* set divisor (lo byte) 115200 baud */
    EARLYCON_UART_IOADDR.out8(UART_DLL, 0x18);
    /* set divisor (hi byte) 115200 baud */
    EARLYCON_UART_IOADDR.out8(UART_DLH, 0x00);
    /* disable DLAB */
    EARLYCON_UART_IOADDR.out8(UART_LCR, lcr & !UART_LCR_DLAB);
}

unsafe fn serial_tx_empty() -> bool {
    return EARLYCON_UART_IOADDR.in8(5) & 0x20 != 0;
}

unsafe fn serial_chr(chr: u8) -> isize {
    if chr == b'\n' {
        serial_chr(b'\r');
    }

    while !serial_tx_empty() {}

    EARLYCON_UART_IOADDR.out8(0, chr);

    return 1;
}

unsafe fn serial_str(mut s: *const u8) -> isize {
    let mut ret = 0;

    while *s != b'\0' {
        ret += serial_chr(*s);
        s = s.offset(1);
    }

    return ret;
}

unsafe fn earlycon_uart_puts(s: *const u8) -> isize {
    return serial_str(s);
}

unsafe fn earlycon_uart_putc(c: u8) -> isize {
    return serial_chr(c);
}

unsafe fn earlycon_uart_init() {
    serial_init();

    /* Assume a terminal, clear formatting */
    serial_str(b"\033[0m\0".as_ptr());
}

pub static mut EARLYCON_UART: EarlyConsole = EarlyConsole {
    _init: Some(earlycon_uart_init),
    _putc: Some(earlycon_uart_putc),
    _puts: Some(earlycon_uart_puts),
};
