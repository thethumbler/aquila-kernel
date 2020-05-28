use prelude::*;
use fs::*;
use dev::dev::*;
use dev::kdev::*;

use crate::{ISDEV, VNODE_DEV, DEV_MAJOR, DEV_MINOR};

pub unsafe fn vfs_ioctl(vnode: *mut Vnode, request: usize, argp: *mut u8) -> isize {
    //vfs_log(LOG_DEBUG, "vfs_ioctl(vnode=%p, request=%ld, argp=%p)\n", vnode, request, argp);

    /* TODO basic ioctl handling */

    /* invalid request */
    if vnode.is_null() {
        return -EINVAL;
    }

    /* device node */
    if ISDEV!(vnode) {
        return kdev_ioctl(&mut VNODE_DEV!(vnode), request as isize, argp);
    }

    /* invalid request */
    if (*vnode).fs.is_null() {
        return -EINVAL;
    }

    /* operation not supported */
    //if ((*(*vnode).fs).vops.ioctl as *mut u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).ioctl(request as isize, argp);
}

#[repr(C)]
pub struct Winsize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

/*
#define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
#define _IOC_NONE  0U
#define _IOC_WRITE 1U
#define _IOC_READ  2U

#define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
#define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
#define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
#define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
*/

pub const TCGETS:               usize = 0x5401;
pub const TCSETS:               usize = 0x5402;
pub const TCSETSW:              usize = 0x5403;
pub const TCSETSF:              usize = 0x5404;
pub const TCGETA:               usize = 0x5405;
pub const TCSETA:               usize = 0x5406;
pub const TCSETAW:              usize = 0x5407;
pub const TCSETAF:              usize = 0x5408;
pub const TCSBRK:               usize = 0x5409;
pub const TCXONC:               usize = 0x540A;
pub const TCFLSH:               usize = 0x540B;
pub const TIOCEXCL:             usize = 0x540C;
pub const TIOCNXCL:             usize = 0x540D;
pub const TIOCSCTTY:            usize = 0x540E;
pub const TIOCGPGRP:            usize = 0x540F;
pub const TIOCSPGRP:            usize = 0x5410;
pub const TIOCOUTQ:             usize = 0x5411;
pub const TIOCSTI:              usize = 0x5412;
pub const TIOCGWINSZ:           usize = 0x5413;
pub const TIOCSWINSZ:           usize = 0x5414;
pub const TIOCMGET:             usize = 0x5415;
pub const TIOCMBIS:             usize = 0x5416;
pub const TIOCMBIC:             usize = 0x5417;
pub const TIOCMSET:             usize = 0x5418;
pub const TIOCGSOFTCAR:         usize = 0x5419;
pub const TIOCSSOFTCAR:         usize = 0x541A;
pub const FIONREAD:             usize = 0x541B;
pub const TIOCINQ:              usize = FIONREAD;
pub const TIOCLINUX:            usize = 0x541C;
pub const TIOCCONS:             usize = 0x541D;
pub const TIOCGSERIAL:          usize = 0x541E;
pub const TIOCSSERIAL:          usize = 0x541F;
pub const TIOCPKT:              usize = 0x5420;
pub const FIONBIO:              usize = 0x5421;
pub const TIOCNOTTY:            usize = 0x5422;
pub const TIOCSETD:             usize = 0x5423;
pub const TIOCGETD:             usize = 0x5424;
pub const TCSBRKP:              usize = 0x5425;
pub const TIOCTTYGSTRUCT:       usize = 0x5426;
pub const TIOCSBRK:             usize = 0x5427;
pub const TIOCCBRK:             usize = 0x5428;
pub const TIOCGSID:             usize = 0x5429;
pub const TIOCGPTN:             usize = 0x80045430;
pub const TIOCSPTLCK:           usize = 0x40045431;
pub const TCGETX:               usize = 0x5432;
pub const TCSETX:               usize = 0x5433;
pub const TCSETXF:              usize = 0x5434;
pub const TCSETXW:              usize = 0x5435;

pub const FIONCLEX:             usize = 0x5450;
pub const FIOCLEX:              usize = 0x5451;
pub const FIOASYNC:             usize = 0x5452;
pub const TIOCSERCONFIG:        usize = 0x5453;
pub const TIOCSERGWILD:         usize = 0x5454;
pub const TIOCSERSWILD:         usize = 0x5455;
pub const TIOCGLCKTRMIOS:       usize = 0x5456;
pub const TIOCSLCKTRMIOS:       usize = 0x5457;
pub const TIOCSERGSTRUCT:       usize = 0x5458;
pub const TIOCSERGETLSR:        usize = 0x5459;
pub const TIOCSERGETMULTI:      usize = 0x545A;
pub const TIOCSERSETMULTI:      usize = 0x545B;

pub const TIOCMIWAIT:           usize = 0x545C;
pub const TIOCGICOUNT:          usize = 0x545D;
pub const TIOCGHAYESESP:        usize = 0x545E;
pub const TIOCSHAYESESP:        usize = 0x545F;
pub const FIOQSIZE:             usize = 0x5460;

pub const TIOCPKT_DATA:         usize = 0;
pub const TIOCPKT_FLUSHREAD:    usize = 1;
pub const TIOCPKT_FLUSHWRITE:   usize = 2;
pub const TIOCPKT_STOP:         usize = 4;
pub const TIOCPKT_START:        usize = 8;
pub const TIOCPKT_NOSTOP:       usize = 16;
pub const TIOCPKT_DOSTOP:       usize = 32;
pub const TIOCPKT_IOCTL:        usize = 64;

pub const TIOCSER_TEMT:         usize = 0x01;

pub const TIOCM_LE:             usize = 0x001;
pub const TIOCM_DTR:            usize = 0x002;
pub const TIOCM_RTS:            usize = 0x004;
pub const TIOCM_ST:             usize = 0x008;
pub const TIOCM_SR:             usize = 0x010;
pub const TIOCM_CTS:            usize = 0x020;
pub const TIOCM_CAR:            usize = 0x040;
pub const TIOCM_RNG:            usize = 0x080;
pub const TIOCM_DSR:            usize = 0x100;
pub const TIOCM_CD:             usize = TIOCM_CAR;
pub const TIOCM_RI:             usize = TIOCM_RNG;
pub const TIOCM_OUT1:           usize = 0x2000;
pub const TIOCM_OUT2:           usize = 0x4000;
pub const TIOCM_LOOP:           usize = 0x8000;
pub const TIOCM_MODEM_BITS:     usize = TIOCM_OUT2;

pub const N_TTY:                usize = 0;
pub const N_SLIP:               usize = 1;
pub const N_MOUSE:              usize = 2;
pub const N_PPP:                usize = 3;
pub const N_STRIP:              usize = 4;
pub const N_AX25:               usize = 5;
pub const N_X25:                usize = 6;
pub const N_6PACK:              usize = 7;
pub const N_MASC:               usize = 8;
pub const N_R3964:              usize = 9;
pub const N_PROFIBUS_FDL:       usize = 10;
pub const N_IRDA:               usize = 11;
pub const N_SMSBLOCK:           usize = 12;
pub const N_HDLC:               usize = 13;
pub const N_SYNC_PPP:           usize = 14;
pub const N_HCI:                usize = 15;

pub const FIOSETOWN:            usize = 0x8901;
pub const SIOCSPGRP:            usize = 0x8902;
pub const FIOGETOWN:            usize = 0x8903;
pub const SIOCGPGRP:            usize = 0x8904;
pub const SIOCATMARK:           usize = 0x8905;
pub const SIOCGSTAMP:           usize = 0x8906;

pub const SIOCADDRT:            usize = 0x890B;
pub const SIOCDELRT:            usize = 0x890C;
pub const SIOCRTMSG:            usize = 0x890D;

pub const SIOCGIFNAME:          usize = 0x8910;
pub const SIOCSIFLINK:          usize = 0x8911;
pub const SIOCGIFCONF:          usize = 0x8912;
pub const SIOCGIFFLAGS:         usize = 0x8913;
pub const SIOCSIFFLAGS:         usize = 0x8914;
pub const SIOCGIFADDR:          usize = 0x8915;
pub const SIOCSIFADDR:          usize = 0x8916;
pub const SIOCGIFDSTADDR:       usize = 0x8917;
pub const SIOCSIFDSTADDR:       usize = 0x8918;
pub const SIOCGIFBRDADDR:       usize = 0x8919;
pub const SIOCSIFBRDADDR:       usize = 0x891a;
pub const SIOCGIFNETMASK:       usize = 0x891b;
pub const SIOCSIFNETMASK:       usize = 0x891c;
pub const SIOCGIFMETRIC:        usize = 0x891d;
pub const SIOCSIFMETRIC:        usize = 0x891e;
pub const SIOCGIFMEM:           usize = 0x891f;
pub const SIOCSIFMEM:           usize = 0x8920;
pub const SIOCGIFMTU:           usize = 0x8921;
pub const SIOCSIFMTU:           usize = 0x8922;
pub const SIOCSIFHWADDR:        usize = 0x8924;
pub const SIOCGIFENCAP:         usize = 0x8925;
pub const SIOCSIFENCAP:         usize = 0x8926;
pub const SIOCGIFHWADDR:        usize = 0x8927;
pub const SIOCGIFSLAVE:         usize = 0x8929;
pub const SIOCSIFSLAVE:         usize = 0x8930;
pub const SIOCADDMULTI:         usize = 0x8931;
pub const SIOCDELMULTI:         usize = 0x8932;
pub const SIOCGIFINDEX:         usize = 0x8933;
pub const SIOGIFINDEX:          usize = SIOCGIFINDEX;
pub const SIOCSIFPFLAGS:        usize = 0x8934;
pub const SIOCGIFPFLAGS:        usize = 0x8935;
pub const SIOCDIFADDR:          usize = 0x8936;
pub const SIOCSIFHWBROADCAST:   usize = 0x8937;
pub const SIOCGIFCOUNT:         usize = 0x8938;

pub const SIOCGIFBR:            usize = 0x8940;
pub const SIOCSIFBR:            usize = 0x8941;

pub const SIOCGIFTXQLEN:        usize = 0x8942;
pub const SIOCSIFTXQLEN:        usize = 0x8943;

pub const SIOCDARP:             usize = 0x8953;
pub const SIOCGARP:             usize = 0x8954;
pub const SIOCSARP:             usize = 0x8955;

pub const SIOCDRARP:            usize = 0x8960;
pub const SIOCGRARP:            usize = 0x8961;
pub const SIOCSRARP:            usize = 0x8962;

pub const SIOCGIFMAP:           usize = 0x8970;
pub const SIOCSIFMAP:           usize = 0x8971;

pub const SIOCADDDLCI:          usize = 0x8980;
pub const SIOCDELDLCI:          usize = 0x8981;

pub const SIOCDEVPRIVATE:       usize = 0x89F0;
pub const SIOCPROTOPRIVATE:     usize = 0x89E0;
