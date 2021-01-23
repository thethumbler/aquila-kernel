use prelude::*;
use fs::*;
use mm::*;

use bits::dirent::DirectoryEntry;
use bits::fcntl::*;
use sys::syscall::file::{FileDescriptor, FileBackend};

use crate::kern::print::cstr;

use crate::{malloc_define, malloc_declare, print};

malloc_define!(M_VNODE, b"vnode\0", b"vnode structure\0");
malloc_define!(M_VFS_PATH, b"vfs-path\0", b"vfs path structure\0");
malloc_define!(M_VFS_NODE, b"vfs-node\0", b"vfs node structure\0");
malloc_define!(M_FS_LIST, b"fs-list\0", b"filesystems list\0");

/** list of registered filesystems */
pub static mut REGISTERED_FS: Vec<Arc<Filesystem>> = Vec::new();
static mut VFSBIND: Vec<(String, Arc<Node>)> = Vec::new();

#[derive(Debug)]
pub struct UserOp<'a> {
    /* root directory */
    pub root:   &'a str,

    /* current working directory */
    pub cwd:    &'a str,

    pub uid:    uid_t,
    pub gid:    gid_t,
    pub mask:   mode_t,
    pub flags:  usize,
}

impl Default for UserOp<'_> {
    fn default() -> Self {
        UserOp {
            root: "/",
            cwd:  "/",
            uid: 0,
            gid: 0,
            mask: 0,
            flags: 0,
        }
    }
}

#[derive(Clone)]
pub struct NodeOps {
    pub read:    Option<fn(node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error>>,
    pub write:   Option<fn(node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error>>,
    pub ioctl:   Option<fn(node: &Node, request: usize, argp: *mut u8) -> Result<usize, Error>>,
    pub close:   Option<fn(node: &Node) -> Result<usize, Error>>,
    pub trunc:   Option<fn(node: &Node, len: usize) -> Result<usize, Error>>,
    pub chmod:   Option<fn(node: &Node, mode: mode_t) -> Result<mode_t, Error>>,
    pub chown:   Option<fn(node: &Node, owner: uid_t, group: gid_t) -> Result<(uid_t, gid_t), Error>>,

    pub readdir: Option<fn(dir: &Node, offset: usize) -> Result<(usize, DirectoryEntry), Error>>,
    pub finddir: Option<fn(dir: &Node, name: &str) -> Result<DirectoryEntry, Error>>,

    pub mknod:   Option<fn(dir: &Node, filename: &str, mode: mode_t, dev: dev_t, uio: &UserOp) -> Result<Arc<Node>, Error>>,
    pub unlink:  Option<fn(dir: &Node, filename: &str, uio: &UserOp) -> Result<(), Error>>,

    pub iget:    Option<fn(superblock: &Node, ino: ino_t) -> Result<&'static Node, Error>>,
    pub iput:    Option<fn(superblock: &Node, node: &mut Node) -> Result<(), Error>>,

    pub vsync:   Option<unsafe fn(vnode: *mut Node, mode: isize) -> isize>,
    pub sync:    Option<unsafe fn(super_node: *mut Node, mode: isize) -> isize>,

    pub map:     Option<unsafe fn(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> isize>,
}

impl NodeOps {
    pub const fn none() -> NodeOps {
        NodeOps {
            read:    None,
            write:   None,
            ioctl:   None,
            close:   None,
            trunc:   None,
            chmod:   None,
            chown:   None,
            readdir: None,
            finddir: None,
            mknod:   None,
            unlink:  None,
            iget:    None,
            iput:    None,
            vsync:   None,
            sync:    None,
            map:     None,
        }
    }
}

pub struct Path<'a> {
    pub node: &'a Node, // FIXME
    pub path: &'a str,
}

/** filesystem structure */
#[derive(Clone)]
pub struct Filesystem {
    pub name: &'static str,

    pub init:    Option<fn() -> Result<(), Error>>,
    pub load:    Option<fn(dev: Arc<Node>) -> Result<Arc<Node>, Error>>,
    pub mount:   Option<fn(fs: Arc<Filesystem>, dir: &str, flags: isize, data: *mut u8) -> Result<(), Error>>,

    pub(in fs) read:    Option<fn(node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error>>,
    pub(in fs) write:   Option<fn(node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error>>,
    pub(in fs) ioctl:   Option<fn(node: &Node, request: usize, argp: *mut u8) -> Result<usize, Error>>,
    pub(in fs) close:   Option<fn(node: &Node) -> Result<usize, Error>>,
    pub(in fs) trunc:   Option<fn(node: &Node, len: usize) -> Result<usize, Error>>,
    pub(in fs) chmod:   Option<fn(node: &Node, mode: mode_t) -> Result<mode_t, Error>>,
    pub(in fs) chown:   Option<fn(node: &Node, owner: uid_t, group: gid_t) -> Result<(uid_t, gid_t), Error>>,

    pub readdir: Option<fn(dir: &Node, offset: usize) -> Result<(usize, DirectoryEntry), Error>>,
    pub finddir: Option<fn(dir: &Node, name: &str) -> Result<DirectoryEntry, Error>>,

    pub(in fs) mknod:   Option<fn(dir: &Node, filename: &str, mode: mode_t, dev: dev_t, uio: &UserOp) -> Result<Arc<Node>, Error>>,
    pub(in fs) unlink:  Option<fn(dir: &Node, filename: &str, uio: &UserOp) -> Result<(), Error>>,

    pub iget:    Option<fn(superblock: &mut Node, ino: ino_t) -> Result<&'static mut Node, Error>>,
    pub iput:    Option<fn(superblock: &mut Node, node: &mut Node) -> Result<(), Error>>,

    pub vsync:   Option<unsafe fn(vnode: *mut Node, mode: isize) -> isize>,
    pub sync:    Option<unsafe fn(super_node: *mut Node, mode: isize) -> isize>,

    pub map:     Option<unsafe fn(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> isize>,
    
    pub fops: FileOps,

    /* flags */
    pub nodev: isize,
}

impl Filesystem {
    pub const fn none() -> Filesystem {
        Filesystem {
            name:    "",
            init:    None,
            load:    None,
            mount:   None,
            read:    None,
            write:   None,
            ioctl:   None,
            close:   None,
            trunc:   None,
            chmod:   None,
            chown:   None,
            readdir: None,
            finddir: None,
            mknod:   None,
            unlink:  None,
            iget:    None,
            iput:    None,
            vsync:   None,
            sync:    None,
            map:     None,
            fops:  FileOps::none(),
            nodev: 0,
        }
    }
}

unsafe impl Sync for Filesystem {}

impl Filesystem {
    // Filesystem operations
    pub fn init(&self) -> Result<(), Error> {
        match self.init {
            Some(f) => f(),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn load(&self, dev: Arc<Node>) -> Result<Arc<Node>, Error> {
        match self.load {
            Some(f) => f(dev),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn mount(&self, _fs: Arc<Filesystem>, dir: &str, flags: isize, data: *mut u8) -> Result<(), Error> {
        match self.mount {
            Some(f) => f(_fs, dir, flags, data),
            None => Err(Error::ENOSYS)
        }
    }

    // Node operations
    pub fn read(&self, node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
        match self.read {
            Some(f) => f(node, offset, size, buffer),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn write(&self, node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
        match self.write {
            Some(f) => f(node, offset, size, buffer),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn readdir(&self, node: &Node, offset: usize) -> Result<(usize, DirectoryEntry), Error> {
        match self.readdir {
            Some(f) => f(node, offset),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn finddir(&self, node: &Node, name: &str) -> Result<DirectoryEntry, Error> {
        match self.finddir {
            Some(f) => f(node, name),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn ioctl(&self, node: &Node, request: usize, argp: *mut u8) -> Result<usize, Error> {
        match self.ioctl {
            Some(f) => f(node, request, argp),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn trunc(&self, node: &Node, len: usize) -> Result<usize, Error> {
        match self.trunc {
            Some(f) => f(node, len),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn chmod(&self, node: &Node, mode: mode_t) -> Result<mode_t, Error> {
        match self.chmod {
            Some(f) => f(node, mode),
            None => Err(Error::ENOSYS)
        }
    }

    pub fn chown(&self, node: &Node, uid: uid_t, gid: gid_t) -> Result<(uid_t, gid_t), Error> {
        match self.chown {
            Some(f) => f(node, uid, gid),
            None => Err(Error::ENOSYS)
        }
    }
}

pub unsafe fn __vfs_can_always(_f: *mut FileDescriptor, _s: usize) -> isize { 1 }
pub unsafe fn __vfs_can_never (_f: *mut FileDescriptor, _s: usize) -> isize { 0 }
pub unsafe fn __vfs_eof_always(_f: *mut FileDescriptor) -> isize { 1 }
pub unsafe fn __vfs_eof_never (_f: *mut FileDescriptor) -> isize { 0 }

pub fn realpath(path: &str, uio: &UserOp) -> Result<String, Error> {
    let fullpath = if path.starts_with('/') {
        path.to_owned()
    } else {
        uio.cwd.to_owned() + "/" + path
    };

    let mut result = Vec::new();

    fullpath
        .split('/')
        .for_each(|s| {
            match s {
                "" => {},
                "." => {},
                ".." => { result.pop(); },
                _ => result.push(s),
            }
        });

    Ok("/".to_owned() + &result.join("/"))
}

pub fn mountpoint(path: &str) -> Result<Path, Error> {
    unsafe {
        if let Some((root, node)) = VFSBIND.iter().find(|s| path.starts_with(&s.0)) {
            Ok(Path {
                node: &**node,
                path: &path[root.len()..],
            })
        } else {
            // FIXME
            Err(Error::EINVAL)
        }
    }
}

pub fn bind(path: &str, target: Arc<Node>) -> Result<(), Error> {
    unsafe {
        // TODO check for existence

        VFSBIND.push((path.to_owned(), target));
        VFSBIND.sort_by(|(path1, _), (path2, _)| path2.cmp(path1));
        
        Ok(())
    }
}

pub unsafe fn vfs_init() {
    //vfs_log(LOG_INFO, "initializing\n");
}

pub fn install(fs: Arc<Filesystem>) -> Result<(), Error> {
    unsafe {
        print!("vfs: registered filesystem {}\n", fs.name);
        REGISTERED_FS.push(fs);
        Ok(())
    }
}

/* ================== VFS high level mappings ================== */
pub unsafe fn vfs_perms_check(file: *mut FileDescriptor, uio: *mut UserOp) -> isize {
    if (*uio).uid == 0 {
        /* root */
        return 0;
    }

    let mut read_perms = false;
    let mut write_perms = false;
    let mut exec_perms = false;

    let mode = (*(*file).backend.vnode).mode();
    let uid  = (*(*file).backend.vnode).uid();
    let gid  = (*(*file).backend.vnode).gid();

    /* read permissions */
    read_perms = if (*file).flags & O_ACCMODE == O_RDONLY && (*file).flags & O_ACCMODE != O_WRONLY {
        if      uid == (*uio).uid  { mode & S_IRUSR != 0 }
        else if gid == (*uio).gid  { mode & S_IRGRP != 0 }
        else                       { mode & S_IROTH != 0 }
    } else {
        true
    };

    /* write permissions */
    write_perms = if (*file).flags & (O_WRONLY | O_RDWR) != 0 {
        if      uid == (*uio).uid { mode & S_IWUSR != 0 }
        else if gid == (*uio).gid { mode & S_IWGRP != 0 }
        else                      { mode & S_IWOTH != 0 }
    } else {
        true
    };

    /* execute permissions */
    exec_perms = if (*file).flags & O_EXEC != 0 {
        if      uid == (*uio).uid { mode & S_IXUSR != 0 }
        else if gid == (*uio).gid { mode & S_IXGRP != 0 }
        else                      { mode & S_IXOTH != 0 }
    } else {
        true
    };
    
    if read_perms && write_perms && exec_perms { 0 } else { -EACCES }
}
