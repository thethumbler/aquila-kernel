use prelude::*;
use fs::*;
use bits::dirent::*;
use sys::thread::Thread;
use dev::kdev::*;
use dev::vnode_dev;

use crate::mm::*;

/** in-core inode structure */
pub struct Node {
    pub(in fs) ino:    ino_t,
    pub(in fs) size:   AtomicUsize,
    pub(in fs) dev:    dev_t,
    pub(in fs) rdev:   dev_t,
    pub(in fs) mode:   AtomicU32,
    pub(in fs) uid:    AtomicU32,
    pub(in fs) gid:    AtomicU32,
    pub(in fs) nlink:  AtomicU32,

    pub atime:  _time_t,
    pub mtime:  _time_t,
    pub ctime:  _time_t,

    /** filesystem implementation */
    pub fs: Option<Arc<Filesystem>>,

    /** filesystem handler private data */
    pub data: AtomicPtr<u8>,

    /** number of processes referencing this vnode */
    pub refcnt: usize,

    pub read_queue: Option<Box<Queue<*mut Thread>>>,
    pub write_queue: Option<Box<Queue<*mut Thread>>>,

    /** virtual memory object associated with vnode */
    pub vm_object: *mut VmObject,
}

impl Node {
    pub const fn none() -> Self {
        Node {
            ino:    0,
            size:   AtomicUsize::new(0),
            dev:    0,
            rdev:   0,
            mode:   AtomicU32::new(0),
            uid:    AtomicU32::new(0),
            gid:    AtomicU32::new(0),
            nlink:  AtomicU32::new(0),
            atime:  TimeSpec::none(),
            mtime:  TimeSpec::none(),
            ctime:  TimeSpec::none(),
            fs: None,
            data: AtomicPtr::new(core::ptr::null_mut()),
            refcnt: 0,
            read_queue: None,
            write_queue: None,
            vm_object: core::ptr::null_mut(),
        }
    }
}

pub enum NodeType {
    Regular,
    Directory,
    Fifo,
    ChrDev,
    BlkDev,
    Link,
    Socket,
}

impl Node {
    pub fn data<T>(&self) -> Option<&mut T> {
        let ptr = self.data.load(SeqCst) as *mut T;
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(&mut *ptr) }
        }
    }

    pub fn set_data<T>(&self, ptr: *mut T) {
        self.data.store(ptr as *mut u8, SeqCst);
    }

    pub fn size(&self) -> usize {
        self.size.load(SeqCst)
    }

    pub fn set_size(&self, size: usize) {
        self.size.store(size, SeqCst);
    }

    pub fn mode(&self) -> mode_t {
        self.mode.load(SeqCst)
    }

    pub fn set_mode(&self, mode: mode_t) {
        self.mode.store(mode, SeqCst);
    }

    pub fn uid(&self) -> uid_t {
        self.uid.load(SeqCst)
    }

    pub fn set_uid(&self, uid: uid_t) {
        self.uid.store(uid, SeqCst);
    }

    pub fn gid(&self) -> gid_t {
        self.gid.load(SeqCst)
    }

    pub fn set_gid(&self, gid: gid_t) {
        self.gid.store(gid, SeqCst);
    }

    pub fn nlink(&self) -> nlink_t {
        self.nlink.load(SeqCst)
    }

    pub fn set_nlink(&self, nlink: nlink_t) {
        self.nlink.store(nlink, SeqCst);
    }

    pub fn node_type(&self) -> NodeType {
        match self.mode() & S_IFMT {
            S_IFSOCK => NodeType::Socket,
            S_IFLNK  => NodeType::Link,
            S_IFREG  => NodeType::Regular,
            S_IFBLK  => NodeType::BlkDev,
            S_IFDIR  => NodeType::Directory,
            S_IFCHR  => NodeType::ChrDev,
            S_IFIFO  => NodeType::Fifo,
            _        => panic!("invalid file mode"),
        }
    }

    pub fn rdev(&self) -> dev_t {
        self.rdev
    }

    pub fn mknod(&self, filename: &str, mode: mode_t, dev: dev_t, uio: &UserOp) -> Result<Arc<Node>, Error> {
        /* not a directory */
        if !self.is_directory() {
            return Err(Error::ENOTDIR);
        }

        if self.fs.is_none() {
            return Err(Error::ENOSYS);
        }

        match self.fs.as_ref().unwrap().mknod {
            Some(f) => f(self, filename, mode, dev, uio),
            None => Err(Error::ENOSYS),
        }
    }

    pub fn unlink(&mut self, filename: &str, uio: &UserOp) -> Result<(), Error> {
        /* not a directory */
        if !self.is_directory() {
            return Err(Error::ENOTDIR);
        }

        if self.fs.is_none() {
            return Err(Error::ENOSYS);
        }

        match self.fs.as_ref().unwrap().unlink {
            Some(f) => f(self, filename, uio),
            None => Err(Error::ENOSYS),
        }
    }

    pub fn map(&self, vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> isize {
        if self.fs.is_none() {
            return -ENOSYS;
        }

        unsafe {
            match self.fs.as_ref().unwrap().map {
                Some(f) => f(vm_space, vm_entry),
                None => -ENOSYS
            }
        }
    }

    pub fn is_device(&self) -> bool {
        S_ISCHR!(self.mode()) || S_ISBLK!(self.mode())
    }

    pub fn is_symlink(&self) -> bool {
        S_ISLNK!(self.mode())
    }

    pub fn is_directory(&self) -> bool {
        S_ISDIR!(self.mode())
    }
}

impl Node {
    pub fn read(&self, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
        match self.node_type() {
            NodeType::Link |
            NodeType::Regular => self.fs.as_ref().unwrap().read(self, offset, size, buffer),
            NodeType::ChrDev |
            NodeType::BlkDev => {
                unsafe {
                    Error::wrap_isize_to_usize(kdev_read(&mut vnode_dev!(self), offset as isize, size, buffer))
                }
            },
            _ => Err(Error::EINVAL)
        }
    }

    pub fn write(&self, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
        match self.node_type() {
            NodeType::Link |
            NodeType::Regular => self.fs.as_ref().unwrap().write(self, offset, size, buffer),
            NodeType::ChrDev |
            NodeType::BlkDev => {
                unsafe {
                    Error::wrap_isize_to_usize(kdev_write(&mut vnode_dev!(self), offset as isize, size, buffer))
                }
            },
            _ => Err(Error::EINVAL)
        }
    }

    pub fn trunc(&self, len: usize) -> Result<usize, Error> {
        match self.node_type() {
            NodeType::Regular => self.fs.as_ref().unwrap().trunc(self, len),
            _ => Err(Error::EINVAL)
        }
    }

    pub fn chown(&self, uid: uid_t, gid: gid_t) -> Result<(uid_t, gid_t), Error> {
        self.fs.as_ref().unwrap().chown(self, uid, gid)
    }

    pub fn chmod(&self, mode: mode_t) -> Result<mode_t, Error> {
        self.fs.as_ref().unwrap().chmod(self, mode)
    }

    pub fn readdir(&self, offset: usize) -> Result<(usize, DirectoryEntry), Error> {
        match self.node_type() {
            NodeType::Directory => self.fs.as_ref().unwrap().readdir(self, offset),
            _ => Err(Error::ENOTDIR)
        }
    }

    pub fn finddir(&self, name: &str) -> Result<DirectoryEntry, Error> {
        match self.node_type() {
            NodeType::Directory => self.fs.as_ref().unwrap().finddir(self, name),
            _ => Err(Error::ENOTDIR)
        }
    }

    /* sync the metadata and/or data associated with a node */
    pub fn sync(&self, mode: isize) -> Result<(), Error> {
        Err(Error::ENOTSUP)
    }
}
