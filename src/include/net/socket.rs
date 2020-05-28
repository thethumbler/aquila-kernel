use prelude::*;

use crate::include::fs::vfs::*;

pub type socklen_t = u32;
pub type sa_family_t = u32;

#[repr(C)]
pub struct SocketAddress {
    pub sa_family: sa_family_t,
    pub sa_data: [u8; 0],
}

#[repr(C)]
pub struct Socket {
    pub domain: isize,
    pub sock_type: isize,
    pub protocol: isize,

    /* socket handler */
    pub ops: *mut SocketOps,

    /* private data */
    pub p: *mut u8,

    pub refcnt: isize,
}

#[repr(C)]
pub struct SocketOps {
    pub accept:  Option<fn(socket: *mut FileDescriptor, conn: *mut FileDescriptor, addr: *const SocketAddress, len: *mut socklen_t) -> isize>,
    pub bind:    Option<fn(socket: *mut FileDescriptor, addr: *const SocketAddress, len: socklen_t) -> isize>,
    pub connect: Option<fn(socket: *mut FileDescriptor, addr: *const SocketAddress, len: socklen_t) -> isize>,
    pub listen:  Option<fn(socket: *mut FileDescriptor, backlog: isize) -> isize>,

    pub recv:    Option<fn(socket: *mut FileDescriptor, buf: *mut u8, len: usize, flags: isize) -> isize>,
    pub send:    Option<fn(socket: *mut FileDescriptor, buf: *mut u8, len: usize, flags: isize) -> isize>,

    pub can_read:   Option<fn(socket: *mut FileDescriptor, len: usize) -> isize>,
    pub can_write:  Option<fn(socket: *mut FileDescriptor, len: usize) -> isize>,

    pub shutdown:   Option<fn(socket: *mut FileDescriptor, how: isize) -> isize>,
}

pub const SOCK_DGRAM      : usize = 0x0001;
pub const SOCK_RAW        : usize = 0x0002;
pub const SOCK_SEQPACKET  : usize = 0x0003;
pub const SOCK_STREAM     : usize = 0x0004;

pub const SOMAXCONN       : usize = 1024;

pub const AF_INET         : usize = 0x0001;
pub const AF_INET6        : usize = 0x0002;
pub const AF_UNIX         : usize = 0x0003;
pub const AF_UNSPEC       : usize = 0x0004;

pub const FILE_SOCKET     : usize = 0x80000000;

pub const MSG_CTRUNC      : usize = 0x0001;
pub const MSG_DONTROUTE   : usize = 0x0002;
pub const MSG_EOR         : usize = 0x0004;
pub const MSG_OOB         : usize = 0x0008;
pub const MSG_NOSIGNAL    : usize = 0x0010;
pub const MSG_PEEK        : usize = 0x0020;
pub const MSG_TRUNC       : usize = 0x0040;
pub const MSG_WAITALL     : usize = 0x0080;

pub const SHUT_RD         : usize = 0x0001;
pub const SHUT_WR         : usize = 0x0002;
pub const SHUT_RDWR       : usize = (SHUT_RD|SHUT_WR);
