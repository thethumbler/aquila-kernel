use prelude::*;

use fs::*;
use dev::*;

pub fn stat(node: &Node) -> Result<Stat, Error> {
    Ok(Stat {
        st_dev   : node.dev,
        st_ino   : node.ino as u16,
        st_mode  : node.mode(),
        st_nlink : node.nlink() as u16,
        st_uid   : node.uid(),
        st_gid   : node.gid(),
        st_rdev  : node.rdev,
        st_size  : node.size() as u32,
        st_mtime : node.mtime,
        st_atime : node.atime,
        st_ctime : node.ctime,
        // FIXME
        st_blksize: 0,
        st_blocks: 0,
    })
}

#[repr(C)]
pub struct Stat {
    pub st_dev: u16,
    pub st_ino: u16,
    pub st_mode: u32,
    pub st_nlink: u16,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u16,
    pub st_size: u32,
    
    pub st_atime: TimeSpec,
    pub st_mtime: TimeSpec,
    pub st_ctime: TimeSpec,
    
    pub st_blksize: u32,
    pub st_blocks: u32,
}

pub const S_IFMT      : mode_t = 0o170000; /* type of file */
pub const S_IFSOCK    : mode_t = 0o140000; /* socket */
pub const S_IFLNK     : mode_t = 0o120000; /* symbolic link */
pub const S_IFREG     : mode_t = 0o100000; /* regular */
pub const S_IFBLK     : mode_t = 0o060000; /* block special */
pub const S_IFDIR     : mode_t = 0o040000; /* directory */
pub const S_IFCHR     : mode_t = 0o020000; /* character special */
pub const S_IFIFO     : mode_t = 0o010000; /* fifo */

pub const S_ENFMT     : mode_t = 0o002000; /* enforcement-mode locking */
pub const S_ISUID     : mode_t = 0o004000; /* set user id on execution */
pub const S_ISGID     : mode_t = 0o002000; /* set group id on execution */
pub const S_ISVTX     : mode_t = 0o001000; /* sticky bit */

pub const S_IREAD     : mode_t = 0o000400; /* read permission, owner */
pub const S_IWRITE    : mode_t = 0o000200; /* write permission, owner */
pub const S_IEXEC     : mode_t = 0o000100; /* execute/search permission, owner */

pub const S_IRUSR     : mode_t = 0o000400; /* read permission, owner */
pub const S_IWUSR     : mode_t = 0o000200; /* write permission, owner */
pub const S_IXUSR     : mode_t = 0o000100; /* execute/search permission, owner */
pub const S_IRWXU     : mode_t = (S_IRUSR | S_IWUSR | S_IXUSR);

pub const S_IRGRP     : mode_t = 0o000040; /* read permission, group */
pub const S_IWGRP     : mode_t = 0o000020; /* write permission, grougroup */
pub const S_IXGRP     : mode_t = 0o000010; /* execute/search permission, group */
pub const S_IRWXG     : mode_t = (S_IRGRP | S_IWGRP | S_IXGRP);

pub const S_IROTH     : mode_t = 0o000004; /* read permission, other */
pub const S_IWOTH     : mode_t = 0o000002; /* write permission, other */
pub const S_IXOTH     : mode_t = 0o000001; /* execute/search permission, other */
pub const S_IRWXO     : mode_t = (S_IROTH | S_IWOTH | S_IXOTH);

pub macro S_ISSOCK { ($n:expr) => { ((($n) & S_IFMT) == S_IFSOCK) } }
pub macro S_ISLNK  { ($n:expr) => { ((($n) & S_IFMT) == S_IFLNK)  } }
pub macro S_ISREG  { ($n:expr) => { ((($n) & S_IFMT) == S_IFREG)  } }
pub macro S_ISBLK  { ($n:expr) => { ((($n) & S_IFMT) == S_IFBLK)  } }
pub macro S_ISDIR  { ($n:expr) => { ((($n) & S_IFMT) == S_IFDIR)  } }
pub macro S_ISCHR  { ($n:expr) => { ((($n) & S_IFMT) == S_IFCHR)  } }
pub macro S_ISIFO  { ($n:expr) => { ((($n) & S_IFMT) == S_IFIFO)  } }
