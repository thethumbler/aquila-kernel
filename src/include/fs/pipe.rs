use prelude::*;

use crate::include::fs::vfs::*;

pub const PIPE_BUFLEN: usize = 1024;

/** unix pipe */
pub struct Pipe {
    /** readers reference count */
    pub r_ref: usize,

    /** writers reference count */
    pub w_ref: usize,

    /** ring buffer */
    pub ring: *mut RingBuf,
}
