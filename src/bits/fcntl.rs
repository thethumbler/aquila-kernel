use prelude::*;

pub const O_ACCMODE   : usize = (O_RDONLY|O_WRONLY|O_RDWR);

pub const O_RDONLY    : usize = 0x000000;
pub const O_WRONLY    : usize = 0x000001;
pub const O_RDWR      : usize = 0x000002;
pub const O_APPEND    : usize = 0x000008;
pub const O_CREAT     : usize = 0x000200;
pub const O_TRUNC     : usize = 0x000400;
pub const O_EXCL      : usize = 0x000800;
pub const O_SYNC      : usize = 0x002000;
pub const O_DSYNC     : usize = O_SYNC;
pub const O_RSYNC     : usize = O_SYNC;
pub const O_NONBLOCK  : usize = 0x004000;
pub const O_NOCTTY    : usize = 0x008000;
pub const O_CLOEXEC   : usize = 0x040000;
pub const O_NOFOLLOW  : usize = 0x100000;
pub const O_DIRECTORY : usize = 0x200000;
pub const O_EXEC      : usize = 0x400000 ;
pub const O_SEARCH    : usize = O_EXEC;

/* fcntl(2) requests */
pub const F_DUPFD         : usize = 0;   /* Duplicate fildes */
pub const F_GETFD         : usize = 1;   /* Get fildes flags (close on exec) */
pub const F_SETFD         : usize = 2;   /* Set fildes flags (close on exec) */
pub const F_GETFL         : usize = 3;   /* Get file flags */
pub const F_SETFL         : usize = 4;   /* Set file flags */
pub const F_GETOWN        : usize = 5;   /* Get owner - for ASYNC */
pub const F_SETOWN        : usize = 6;   /* Set owner - for ASYNC */
pub const F_GETLK         : usize = 7;   /* Get record-locking information */
pub const F_SETLK         : usize = 8;   /* Set or Clear a record-lock (Non-Blocking) */
pub const F_SETLKW        : usize = 9;   /* Set or Clear a record-lock (Blocking) */
pub const F_RGETLK        : usize = 10;  /* Test a remote lock to see if it is blocked */
pub const F_RSETLK        : usize = 11;  /* Set or unlock a remote lock */
pub const F_CNVT          : usize = 12;  /* Convert a fhandle to an open fd */
pub const F_RSETLKW       : usize = 13;  /* Set or Clear remote record-lock(Blocking) */
pub const F_DUPFD_CLOEXEC : usize = 14;  /* As F_DUPFD, but set close-on-exec flag */

pub const FD_CLOEXEC      : usize = 1;
