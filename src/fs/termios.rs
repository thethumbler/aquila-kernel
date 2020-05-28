use prelude::*;

pub type cc_t = u8;
pub type speed_t = isize;
pub type tcflag_t = isize;

pub const NCCS: usize = 32;

#[repr(C)]
pub struct Termios {
     pub c_iflag:  tcflag_t,
     pub c_oflag:  tcflag_t,
     pub c_cflag:  tcflag_t,
     pub c_lflag:  tcflag_t,
     pub c_line:   cc_t,
     pub c_cc:     [cc_t; NCCS],
     pub c_ispeed: speed_t,
     pub c_ospeed: speed_t,
}

pub const VINTR:    usize = 0;
pub const VQUIT:    usize = 1;
pub const VERASE:   usize = 2;
pub const VKILL:    usize = 3;
pub const VEOF:     usize = 4;
pub const VTIME:    usize = 5;
pub const VMIN:     usize = 6;
pub const VSWTC:    usize = 7;
pub const VSTART:   usize = 8;
pub const VSTOP:    usize = 9;
pub const VSUSP:    usize = 10;
pub const VEOL:     usize = 11;
pub const VREPRINT: usize = 12;
pub const VDISCARD: usize = 13;
pub const VWERASE:  usize = 14;
pub const VLNEXT:   usize = 15;
pub const VEOL2:    usize = 16;

pub const IGNBRK:  tcflag_t = 0o0000001;
pub const BRKINT:  tcflag_t = 0o0000002;
pub const IGNPAR:  tcflag_t = 0o0000004;
pub const PARMRK:  tcflag_t = 0o0000010;
pub const INPCK:   tcflag_t = 0o0000020;
pub const ISTRIP:  tcflag_t = 0o0000040;
pub const INLCR:   tcflag_t = 0o0000100;
pub const IGNCR:   tcflag_t = 0o0000200;
pub const ICRNL:   tcflag_t = 0o0000400;
pub const IUCLC:   tcflag_t = 0o0001000;
pub const IXON:    tcflag_t = 0o0002000;
pub const IXANY:   tcflag_t = 0o0004000;
pub const IXOFF:   tcflag_t = 0o0010000;
pub const IMAXBEL: tcflag_t = 0o0020000;
pub const IUTF8:   tcflag_t = 0o0040000;

pub const OPOST:  tcflag_t = 0o0000001;
pub const OLCUC:  tcflag_t = 0o0000002;
pub const ONLCR:  tcflag_t = 0o0000004;
pub const OCRNL:  tcflag_t = 0o0000010;
pub const ONOCR:  tcflag_t = 0o0000020;
pub const ONLRET: tcflag_t = 0o0000040;
pub const OFILL:  tcflag_t = 0o0000100;
pub const OFDEL:  tcflag_t = 0o0000200;
pub const NLDLY:  tcflag_t = 0o0000400;
pub const NL0:    tcflag_t = 0o0000000;
pub const NL1:    tcflag_t = 0o0000400;
pub const CRDLY:  tcflag_t = 0o0003000;
pub const CR0:    tcflag_t = 0o0000000;
pub const CR1:    tcflag_t = 0o0001000;
pub const CR2:    tcflag_t = 0o0002000;
pub const CR3:    tcflag_t = 0o0003000;
pub const TABDLY: tcflag_t = 0o0014000;
pub const TAB0:   tcflag_t = 0o0000000;
pub const TAB1:   tcflag_t = 0o0004000;
pub const TAB2:   tcflag_t = 0o0010000;
pub const TAB3:   tcflag_t = 0o0014000;
pub const BSDLY:  tcflag_t = 0o0020000;
pub const BS0:    tcflag_t = 0o0000000;
pub const BS1:    tcflag_t = 0o0020000;
pub const FFDLY:  tcflag_t = 0o0100000;
pub const FF0:    tcflag_t = 0o0000000;
pub const FF1:    tcflag_t = 0o0100000;

pub const VTDLY: usize = 0o0040000;
pub const VT0:   usize = 0o0000000;
pub const VT1:   usize = 0o0040000;

pub const B0:     usize = 0o0000000;
pub const B50:    usize = 0o0000001;
pub const B75:    usize = 0o0000002;
pub const B110:   usize = 0o0000003;
pub const B134:   usize = 0o0000004;
pub const B150:   usize = 0o0000005;
pub const B200:   usize = 0o0000006;
pub const B300:   usize = 0o0000007;
pub const B600:   usize = 0o0000010;
pub const B1200:  usize = 0o0000011;
pub const B1800:  usize = 0o0000012;
pub const B2400:  usize = 0o0000013;
pub const B4800:  usize = 0o0000014;
pub const B9600:  usize = 0o0000015;
pub const B19200: usize = 0o0000016;
pub const B38400: usize = 0o0000017;

pub const B57600:   usize = 0o0010001;
pub const B115200:  usize = 0o0010002;
pub const B230400:  usize = 0o0010003;
pub const B460800:  usize = 0o0010004;
pub const B500000:  usize = 0o0010005;
pub const B576000:  usize = 0o0010006;
pub const B921600:  usize = 0o0010007;
pub const B1000000: usize = 0o0010010;
pub const B1152000: usize = 0o0010011;
pub const B1500000: usize = 0o0010012;
pub const B2000000: usize = 0o0010013;
pub const B2500000: usize = 0o0010014;
pub const B3000000: usize = 0o0010015;
pub const B3500000: usize = 0o0010016;
pub const B4000000: usize = 0o0010017;

pub const CBAUD: usize = 0o0010017;

pub const CSIZE:  usize = 0o0000060;
pub const CS5:    usize = 0o0000000;
pub const CS6:    usize = 0o0000020;
pub const CS7:    usize = 0o0000040;
pub const CS8:    usize = 0o0000060;
pub const CSTOPB: usize = 0o0000100;
pub const CREAD:  usize = 0o0000200;
pub const PARENB: usize = 0o0000400;
pub const PARODD: usize = 0o0001000;
pub const HUPCL:  usize = 0o0002000;
pub const CLOCAL: usize = 0o0004000;

pub const ISIG:   tcflag_t = 0o0000001;
pub const ICANON: tcflag_t = 0o0000002;
pub const ECHO:   tcflag_t = 0o0000010;
pub const ECHOE:  tcflag_t = 0o0000020;
pub const ECHOK:  tcflag_t = 0o0000040;
pub const ECHONL: tcflag_t = 0o0000100;
pub const NOFLSH: tcflag_t = 0o0000200;
pub const TOSTOP: tcflag_t = 0o0000400;
pub const IEXTEN: tcflag_t = 0o0100000;

pub const ECHOCTL: usize = 0o0001000;
pub const ECHOPRT: usize = 0o0002000;
pub const ECHOKE:  usize = 0o0004000;
pub const FLUSHO:  usize = 0o0010000;
pub const PENDIN:  usize = 0o0040000;

pub const TCOOFF: usize = 0;
pub const TCOON:  usize = 1;
pub const TCIOFF: usize = 2;
pub const TCION:  usize = 3;

pub const TCIFLUSH:  usize = 0;
pub const TCOFLUSH:  usize = 1;
pub const TCIOFLUSH: usize = 2;

pub const TCSANOW:   usize = 0;
pub const TCSADRAIN: usize = 1;
pub const TCSAFLUSH: usize = 2;
