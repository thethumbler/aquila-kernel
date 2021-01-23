use prelude::*;

use bits::dirent::*;
use fs::{self, *};
use fs::initramfs;
use fs::posix::*;
use fs::rofs::*;
use kern::time::*;
use mm::*;
use sys::syscall::file::{FileDescriptor, FileBackend};
use alloc::collections::btree_map::BTreeMap;

malloc_declare!(M_VNODE);

const CPIO_BIN_MAGIC: u16 = 0o070707;

#[repr(C)]
#[derive(Default)]
struct CpioHeader {
    magic:    u16,
    dev:      u16,
    ino:      u16,
    mode:     u16,
    uid:      u16,
    gid:      u16,
    nlink:    u16,
    rdev:     u16,
    mtime:    [u16; 2],
    namesize: u16,
    filesize: [u16; 2],
}

impl CpioHeader {
    fn filesize(&self) -> usize {
        self.filesize[0] as usize * 0x10000 + self.filesize[1] as usize
    }

    fn namesize(&self) -> usize {
        self.namesize as usize
    }
}

struct Cpio {
    dev: Arc<Node>,
    dir: Option<BTreeMap<String, Arc<Node>>>,

    /* offset of data in the archive */
    data: usize,
}

malloc_define!(M_CPIO, "cpio\0", "CPIO structure\0");

fn root_node(dev: Arc<Node>) -> Result<Arc<Node>, Error> {
    let mut node = Node::none();

    //node.ino = &*node as *const _ as ino_t;
    node.set_size(0);
    node.set_mode(S_IFDIR | 0o775);
    node.set_uid(0);
    node.set_gid(0);
    node.set_nlink(2);
    node.refcnt = 1;

    let ts = gettime().unwrap();

    node.ctime = ts;
    node.atime = ts;
    node.mtime = ts;

    node.fs = unsafe { CPIOFS_ARC_OPT.clone() };

    let data = Box::new(Cpio {
        dev  : dev,
        dir  : None,
        data : 0,
    });

    node.set_data(Box::leak(data));

    Ok(Arc::new(node))
}

fn new_node(dev: Arc<Node>, name: &str, hdr: &CpioHeader, sz: usize, content: usize) -> Result<Arc<Node>, Error> {
    let mut node = Node::none();

    node.set_size(sz);
    node.set_uid(0);
    node.set_gid(0);
    node.set_mode(hdr.mode as u32);
    node.set_nlink(hdr.nlink as u32);
    node.mtime = TimeSpec { tv_sec: hdr.mtime[0] as u64 * 0x10000 + hdr.mtime[1] as u64, tv_nsec: 0 };
    node.rdev  = hdr.rdev;

    node.fs = unsafe { CPIOFS_ARC_OPT.clone() };

    let data = Box::new(Cpio {
        dev: dev,
        dir: None,
        data: content,
    });

    node.set_data(Box::leak(data));

    Ok(Arc::new(node))
}

fn new_child_node(parent: &Node, name: &str, child: Arc<Node>) -> Result<(), Error> {
    if !parent.is_directory() {
        /* adding child to non directory parent */
        return Err(Error::ENOTDIR);
    }

    let parent_data = parent.data::<Cpio>().unwrap();

    if let None = parent_data.dir {
        parent_data.dir = Some(BTreeMap::new());
    }

    parent_data.dir.as_mut().unwrap()
        .insert(name.to_owned(), child);

    Ok(())
}

fn cpio_find(root: Arc<Node>, path: &str) -> Option<Arc<Node>> {
    let path = fs::realpath(path, &UserOp::default()).unwrap();

    if !root.is_directory() {
        /* not even a directory */
        return None;
    }

    let mut cur = Arc::clone(&root);

    if cur.data::<Cpio>().unwrap().dir.as_ref().is_none() {
        /* directory has no children */
        if path.is_empty() || path == "/" {
            return Some(Arc::clone(&root));
        } else {
            return None;
        }
    }

    for token in path.split("/") {
        if token.is_empty() {
            continue;
        }

        let cpio = cur.data::<Cpio>().unwrap();

        if let Some(node) = cpio.dir.as_ref().unwrap().get(token) {
            cur = Arc::clone(node);
        } else {
            return None;
        }
    }

    Some(cur)
}

fn finddir(root: &Node, name: &str) -> Result<DirectoryEntry, Error> {
    if !root.is_directory() {
        /* not even a directory */
        return Err(Error::ENOTDIR);
    }

    root.data::<Cpio>().map(|cpio|
        cpio.dir.as_ref().map(|dir| {
            dir.get(name)
                // FIXME: do not use pointers as inode numbers
                .map(|node| Ok(DirectoryEntry::new(&**node as *const _ as ino_t, name)))
                .unwrap_or(Err(Error::ENOENT))
        }).unwrap_or(Err(Error::ENOENT))
    ).unwrap_or(Err(Error::ENOENT))
}

fn iget(_superblock: &mut Node, ino: ino_t) -> Result<&'static mut Node, Error> {
    unsafe {
        let node = ino as *mut Node;
        Ok(&mut *node)
    }
}

const MAX_NAMESIZE: usize = 1024;

fn load(dev: Arc<Node>) -> Result<Arc<Node>, Error> {
    unsafe {
        if CPIOFS_ARC_OPT.is_none() {
            CPIOFS_ARC_OPT = Some(Arc::new(CPIOFS.clone()))
        }
    }

    /* allocate the root node */
    let rootfs = root_node(Arc::clone(&dev))?;

    let mut hdr = CpioHeader::default();
    let mut offset = 0;

    while offset < dev.size() {
        let mut data_offset = offset;
        dev.read(data_offset, core::mem::size_of_val(&hdr), &hdr as *const _ as *mut u8)?;

        if hdr.magic != CPIO_BIN_MAGIC {
            panic!("invalid cpio archive");
        }

        let size = hdr.filesize();
        
        data_offset += core::mem::size_of::<CpioHeader>();
        
        let mut path = [0u8; MAX_NAMESIZE];
        dev.read(data_offset, 1024, &path as *const _ as *mut u8)?;

        let path = cstr(path.as_ptr());

        match path {
            /* end of archive */
            "TRAILER!!!" => break,
            "." => {
                offset += (core::mem::size_of::<CpioHeader>() + ((hdr.namesize() + 1)/2*2 + (size+1)/2*2) as usize);
                continue;
            },
            path => {
                let (dir, name) = path.rsplit_once("/").unwrap_or(("", path));
                data_offset += hdr.namesize() + hdr.namesize() % 2;
                let node = new_node(Arc::clone(&dev), name, &mut hdr, size, data_offset).unwrap();

                let parent = cpio_find(Arc::clone(&rootfs), dir).unwrap();
                new_child_node(&*parent, name, node);

                offset += (core::mem::size_of::<CpioHeader>() + ((hdr.namesize() + 1)/2*2 + (size+1)/2*2) as usize);
            }
        }
    }

    Ok(rootfs)
}

fn read(node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
    if offset >= node.size() {
        return Ok(0);
    }

    let mut len = if size < node.size() - offset { size } else { node.size() - offset };
    let data = node.data::<Cpio>().unwrap();

    data.dev.read(data.data + offset, len, buffer)
}

fn readdir(node: &Node, offset: usize) -> Result<(usize, DirectoryEntry), Error> {
    match offset {
        0 => Ok((1, DirectoryEntry::new(0, "."))),
        1 => Ok((1, DirectoryEntry::new(0, ".."))),
        offset => node.data::<Cpio>().map(|cpio| {
            cpio.dir.as_ref().map(|dir| {
                dir.iter().nth(offset-2).map(|(name, node)| {
                    Ok((1, DirectoryEntry::new((**node).ino, name)))
                }).unwrap_or(Ok((0, DirectoryEntry::none())))
            }).unwrap_or(Ok((0, DirectoryEntry::none())))
        }).unwrap_or(Ok((0, DirectoryEntry::none())))
    }
}

fn close(_node: &Node) -> Result<usize, Error> {
    return Ok(0);
}

unsafe fn cpio_eof(file: *mut FileDescriptor) -> isize {
    if (*(*file).backend.vnode).is_directory() {
        let cpio = (*(*file).backend.vnode).data::<Cpio>().unwrap();
        return ((*file).offset >= cpio.dir.as_ref().map(|e| e.len()).unwrap_or(0) as isize) as isize;
    } else {
        return ((*file).offset >= (*(*file).backend.vnode).size() as isize) as isize;
    }
}

fn init() -> Result<(), Error> {
    unsafe {
        initramfs::archiver_register(&mut CPIOFS)
    }
}

static mut CPIOFS: Filesystem = Filesystem {
    name: "cpio",
    nodev: 0,
    load: Some(load),

    read:     Some(read),
    readdir:  Some(readdir),
    finddir:  Some(finddir),
    iget:     Some(iget),
    close:    Some(close),

    write:    Some(rofs::write),
    trunc:    Some(rofs::trunc),
    mknod:    Some(rofs::mknod),
    unlink:   Some(rofs::unlink),
    
    fops: FileOps {
        _open:     Some(posix_file_open),
        //_close:    Some(posix_file_close),

        _eof:      Some(cpio_eof),

        ..FileOps::none()
    },

    ..Filesystem::none()
};

static mut CPIOFS_ARC_OPT: Option<Arc<Filesystem>> = None;

module_define!{
    "initramfs_cpio",
    None,
    Some(init),
    None
}
