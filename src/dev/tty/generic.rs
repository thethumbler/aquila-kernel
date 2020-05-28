use prelude::*;

use kern::string::*;
use crate::include::mm::kvmem::*;
use crate::include::fs::termios::*;
use crate::include::bits::errno::*;
use crate::dev::tty::tty::*;
use crate::include::fs::ioctl::*;
use crate::sys::proc::Process;
use crate::include::core::types::*;
use crate::sys::signal::*;
use crate::{malloc_define, curproc, print};

malloc_define!(M_TTY, b"tty\0", b"tty structure\0");
malloc_define!(M_TTY_COOK, b"tty-cook\0", "tty cooking buffer\0");

pub unsafe fn tty_master_write(tty: *mut Tty, size: usize, buf: *mut u8) -> isize {
    let mut ret = size;
    let mut size = size;

    /* process slave input */
    if (*tty).tios.c_lflag & ICANON != 0 {
        /* canonical mode */
        let echo = (*tty).tios.c_lflag & ECHO != 0;
        let mut c = buf;

        while size != 0 {
            let mut skip_echo = false;

            if *c == (*tty).tios.c_cc[VEOF] {
                /* TODO */
            } else if *c == (*tty).tios.c_cc[VEOL] {
                /* TODO */
            } else if *c == (*tty).tios.c_cc[VERASE] {
                /* the ERASE character shall delete the last character in
                 * the current line, if there is one.
                 */
                if (*tty).pos > 0 {
                    (*tty).pos -= 1;
                    *(*tty).cook.offset((*tty).pos as isize) = b'\0';

                    if (*tty).tios.c_lflag & ECHOE != 0 {
                        tty_slave_write(tty, 3, b"\x08 \x08\0".as_ptr());
                    }
                }

                //goto skip_echo;
                skip_echo = true;
            } else if *c == (*tty).tios.c_cc[VINTR] {
                signal_pgrp_send((*tty).fg, SIGINT);
                let cc = [b'^', *c + b'@', b'\n'];
                tty_slave_write(tty, 3, cc.as_ptr());
                //goto skip_echo;
                skip_echo = true;
            } else if *c == (*tty).tios.c_cc[VKILL] {
                /* The KILL character shall delete all data in the current
                 * line, if there is any.
                 */
            } else if *c == (*tty).tios.c_cc[VQUIT] {
            } else if *c == (*tty).tios.c_cc[VSTART] {
            } else if *c == (*tty).tios.c_cc[VSUSP] {
            } else if *c == b'\n' || (*c == b'\r' && ((*tty).tios.c_iflag & ICRNL != 0)) {
                *(*tty).cook.offset((*tty).pos as isize) = b'\n';
                (*tty).pos += 1;

                if echo {
                    tty_slave_write(tty, 1, b"\n".as_ptr());
                }

                (*tty).slave_write.unwrap()(tty, (*tty).pos, (*tty).cook);
                (*tty).pos = 0;

                ret = ret - size + 1;
                return ret as isize;
            } else {
                *(*tty).cook.offset((*tty).pos as isize) = *c;
                (*tty).pos += 1;
            }

            if echo && !skip_echo {
                if *c < b' ' {
                    /* non-printable */
                    let cc = [ b'^', *c + b'@' ];
                    tty_slave_write(tty, 2, cc.as_ptr());
                } else {
                    tty_slave_write(tty, 1, c);
                }
            }

            c = c.offset(1);
            size -= 1;
        }
    } else {
        return (*tty).slave_write.unwrap()(tty, size, buf);
    }

    return ret as isize;
}

pub unsafe fn tty_slave_write(tty: *mut Tty, size: usize, buf: *const u8) -> isize {
    if (*tty).tios.c_oflag & OPOST != 0 {
        let mut written = 0;

        while written < size {
            let c = *buf.offset(written as isize);

            if c == b'\n' && ((*tty).tios.c_oflag & ONLCR != 0) {
                /* if ONLCR is set, the NL character shall
                 * be transmitted as the CR-NL character pair. 
                 */
                if ((*tty).master_write.unwrap()(tty, 2, b"\r\n".as_ptr()) != 2) {
                    /* FIXME should handle special cases */
                    break;
                }

            } else if c == b'\r' && ((*tty).tios.c_oflag & OCRNL != 0) {
                /* if OCRNL is set, the CR character shall
                 * be transmitted as the NL character.
                 */
                if ((*tty).master_write.unwrap()(tty, 1, b"\n".as_ptr()) != 1) {
                    break;
                }
            } else if c == b'\r' && ((*tty).tios.c_oflag & ONOCR != 0) {
                /* if ONOCR is set, no CR character shall be
                 * transmitted when at column 0 (first position)
                 */
                if (*tty).pos % (*tty).ws.ws_row as usize != 0 {
                    if (*tty).master_write.unwrap()(tty, 1, &c) != 1 {
                        break;
                    }
                }
            } else if c == b'\n' && ((*tty).tios.c_oflag & ONLRET != 0) {
                /* if ONLRET is set, the NL character is assumed
                 * to do the carriage-return function; the column
                 * pointer shall be set to 0 and the delays specified
                 * for CR shall be used. Otherwise, the NL character is
                 * assumed to do just the line-feed function; the column
                 * pointer remains unchanged. The column pointer shall also
                 * be set to 0 if the CR character is actually transmitted.
                 */

                /* TODO */
            } else {
                if (*tty).master_write.unwrap()(tty, 1, &c) != 1 {
                    break;
                }
            }

            written += 1;
        }

        return written as isize;
    } else {
        return (*tty).master_write.unwrap()(tty, size, buf);
    }
}

pub unsafe fn tty_ioctl(tty: *mut Tty, request: isize, argp: *mut u8) -> isize {
    match request as usize {
        TCGETS => {
            memcpy(argp, &((*tty).tios) as *const _ as *mut u8, core::mem::size_of::<Termios>());
        },
        TCSETS => {
            memcpy(&((*tty).tios) as *const _ as *mut u8, argp, core::mem::size_of::<Termios>());
        },
        TIOCGPGRP => {
            *(argp as *mut pid_t) = (*(*tty).fg).pgid;
        },
        TIOCSPGRP => {
            (*tty).fg = (*curproc!()).pgrp;
            /* XXX */ 
        },
        TIOCGWINSZ => {
            memcpy(argp, &(*tty).ws as *const _ as *mut u8, core::mem::size_of::<Winsize>());
        },
        TIOCSWINSZ => {
            memcpy(&(*tty).ws as *const _ as *mut u8, argp, core::mem::size_of::<Winsize>());
        },
        TIOCSCTTY => {
            /* FIXME */
            (*tty).proc = curproc!();
            (*(*(*curproc!()).pgrp).session).ctty = (*tty).dev as *mut u8;
        },
        _ => return -EINVAL,
    };
    
    return 0;
}

/**
 * \ingroup dev-tty
 * \brief create a new generic tty interface
 */
pub unsafe fn tty_new(proc: *mut Process, buf_size: usize, master: ttyio, slave: ttyio, p: *mut u8, tty_ref: *mut *mut Tty) -> isize {
    let tty = kmalloc(core::mem::size_of::<Tty>(), &M_TTY, M_ZERO) as *mut Tty;
    if tty.is_null() {
        return -ENOMEM;
    }

    let mut buf_size = buf_size;

    if buf_size == 0 {
        buf_size = TTY_BUF_SIZE;
    }

    (*tty).cook = kmalloc(buf_size, &M_TTY_COOK, 0);

    if (*tty).cook.is_null() {
        kfree(tty as *mut u8);
        return -ENOMEM;
    }

    (*tty).pos  = 0;

    /* defaults */
    (*tty).tios.c_iflag = ICRNL | IXON;
    (*tty).tios.c_oflag = OPOST | ONLCR;
    (*tty).tios.c_lflag = ISIG | ICANON | ECHO | ECHOE | ECHOK;
    //(*tty).tios.c_cc[VEOL]   = ;
    (*tty).tios.c_cc[VERASE] = 0x08;  /* BS */
    (*tty).tios.c_cc[VINTR]  = 0x03;  /* ^C */
    (*tty).tios.c_cc[VKILL]  = 0x15;  /* ^U */
    (*tty).tios.c_cc[VQUIT]  = 0x1C;  /* ^\ */
    (*tty).tios.c_cc[VSTART] = 0x11;  /* ^Q */
    (*tty).tios.c_cc[VSUSP]  = 0x1A;  /* ^Z */

    (*tty).ws.ws_row = 24;
    (*tty).ws.ws_col = 80;

    (*tty).fg = (*proc).pgrp;

    /* interface */
    (*tty).master_write = master;
    (*tty).slave_write  = slave;
    (*tty).p = p;

    if !tty_ref.is_null() {
        *tty_ref = tty;
    }

    return 0;
}

pub unsafe fn tty_free(tty: *mut Tty) -> isize {
    kfree((*tty).cook);
    kfree(tty as *mut u8);

    return 0;
}
