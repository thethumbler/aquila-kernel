use prelude::*;
use fs::*;
use fs::rofs::*;
use fs::posix::*;
use fs::initramfs::*;
use mm::*;
use bits::dirent::DirectoryEntry;

use kern::string::*;
use kern::time::*;

use crate::{print, malloc_declare, malloc_define};

malloc_declare!(M_VNODE);

const CPIO_BIN_MAGIC: u16 = 0o070707;

#[repr(C)]
struct CpioHeader {
    pub magic:    u16,
    pub dev:      u16,
    pub ino:      u16,
    pub mode:     u16,
    pub uid:      u16,
    pub gid:      u16,
    pub nlink:    u16,
    pub rdev:     u16,
    pub mtime:    [u16; 2],
    pub namesize: u16,
    pub filesize: [u16; 2],
}

struct Cpio {
    pub super_node: *mut Vnode,
    pub parent: *mut Vnode,
    pub dir: *mut Vnode,

    pub count: usize,

    /* offset of data in the archive */
    pub data: usize,

    pub name: *const u8,

    /* for directories */
    pub next: *mut Vnode,
}

malloc_define!(M_CPIO, "cpio\0", "CPIO structure\0");

unsafe fn cpio_root_node(super_node: *mut Vnode, vnode_ref: *mut *mut Vnode) -> isize {
    let mut err = 0;

    let vnode: *mut Vnode = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, M_ZERO) as *mut Vnode;

    if vnode.is_null() {
        err = -ENOMEM;
        panic!("todo");
    }

    let p: *mut Cpio = kmalloc(core::mem::size_of::<Cpio>(), &M_CPIO, 0) as *mut Cpio;
    if p.is_null() {
        err = -ENOMEM;
        panic!("todo");
    }

    (*vnode).ino    = vnode as ino_t;
    (*vnode).size   = 0;
    (*vnode).mode   = S_IFDIR | 0775;
    (*vnode).uid    = 0;
    (*vnode).gid    = 0;
    (*vnode).nlink  = 2;
    (*vnode).refcnt = 1;

    let mut ts: TimeSpec = core::mem::uninitialized();
    gettime(&mut ts);

    (*vnode).ctime = ts;
    (*vnode).atime = ts;
    (*vnode).mtime = ts;

    (*vnode).fs    = &cpiofs;
    (*vnode).p     = p as *mut u8;

    (*p).super_node = super_node;
    (*p).parent     = core::ptr::null_mut();
    (*p).dir        = core::ptr::null_mut();
    (*p).count      = 0;
    (*p).data       = 0;
    (*p).next       = core::ptr::null_mut();

    if !vnode_ref.is_null() {
        *vnode_ref = vnode;
    }

    return 0;
}

unsafe fn cpio_new_node(name: *const u8, hdr: *mut CpioHeader, sz: usize, data: usize, sp: *mut Vnode, vnode_ref: *mut *mut Vnode) -> isize {
    let mut err = 0;

    let vnode: *mut Vnode = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, M_ZERO) as *mut Vnode;

    if vnode.is_null() {
        err = -ENOMEM;
        panic!("todo");
    }

    let p: *mut Cpio = kmalloc(core::mem::size_of::<Cpio>(), &M_CPIO, 0) as *mut Cpio;
    if p.is_null() {
        err = -ENOMEM;
        panic!("todo");
    }

    (*vnode).ino   = vnode as vino_t;
    (*vnode).size  = sz;
    (*vnode).uid   = 0;
    (*vnode).gid   = 0;
    (*vnode).mode  = (*hdr).mode as u32;
    (*vnode).nlink = (*hdr).nlink as u32;
    (*vnode).mtime = TimeSpec { tv_sec: (*hdr).mtime[0] as u64 * 0x10000 + (*hdr).mtime[1] as u64, tv_nsec: 0 };
    (*vnode).rdev  = (*hdr).rdev;

    (*vnode).fs   = &cpiofs;
    (*vnode).p    = p as *mut u8;

    (*p).super_node  = sp;
    (*p).parent = core::ptr::null_mut();
    (*p).dir    = core::ptr::null_mut();
    (*p).count  = 0;
    (*p).data   = data;
    (*p).next   = core::ptr::null_mut();
    (*p).name   = strdup(name);

    if !vnode_ref.is_null() {
        *vnode_ref = vnode;
    }

    return 0;
}

unsafe fn cpio_new_child_node(parent: *mut Vnode, child: *mut Vnode) -> *mut Vnode {
    if parent.is_null() || child.is_null() {
        /* invalid vnode */
        return core::ptr::null_mut();
    }

    if !S_ISDIR!((*parent).mode) {
        /* adding child to non directory parent */
        return core::ptr::null_mut();
    }

    let pp = (*parent).p as *mut Cpio;
    let cp = (*child).p as *mut Cpio;

    let tmp = (*pp).dir;

    (*cp).next = tmp;
    (*pp).dir = child;

    (*pp).count += 1;
    (*cp).parent = parent;

    return child;
}

unsafe fn cpio_find(root: *mut Vnode, path: *const u8) -> *mut Vnode {
    let tokens: *mut *mut u8 = tokenize(path, b'/');

    if !S_ISDIR!((*root).mode) {
        /* not even a directory */
        return core::ptr::null_mut();
    }

    let mut cur = root;
    let mut dir = (*((*cur).p as *mut Cpio)).dir;

    if dir.is_null() {
        /* directory has no children */
        if (*tokens).is_null() {
            return root;
        } else {
            return core::ptr::null_mut();
        }
    }

    let mut token_p = tokens;

    while !(*token_p).is_null() {
        let token = *token_p;

        let mut flag = false;

        let mut node = dir; 

        while !node.is_null() {
            let cpio = (*node).p as *mut Cpio;

            if !(*cpio).name.is_null() && strcmp((*cpio).name, token) == 0 {
                cur = node;
                dir = (*cpio).dir;
                flag = true;
                break;
            }

            node = (*cpio).next;
        }

        if !flag {
            /* no such file or directory */
            return core::ptr::null_mut();
        }

        token_p = token_p.offset(1);
    }

    free_tokens(tokens);
    return cur;
}

unsafe fn cpio_finddir(root: *mut Vnode, name: *const u8, dirent: *mut DirectoryEntry) -> isize {

    if !S_ISDIR!((*root).mode) {
        /* not even a directory */
        return -ENOTDIR;
    }

    let dir = (*((*root).p as *mut Cpio)).dir;

    if dir.is_null() {
        /* directory has no children */
        return -ENOENT;
    }

    let mut node = dir;
    while !node.is_null() {
        let cpio = (*node).p as *mut Cpio;

        if !(*cpio).name.is_null() && strcmp((*cpio).name, name) == 0 {
            if !dirent.is_null() {
                (*dirent).d_ino = node as ino_t;
                strcpy((*dirent).d_name.as_mut_ptr(), name);
            }

            return 0;
        }

        node = (*cpio).next;
    }

    return -ENOENT;
}

unsafe fn cpio_vget(super_node: *mut Vnode, ino: ino_t, vnode_ref: *mut *mut Vnode) -> isize {
    *vnode_ref = ino as *mut Vnode;
    return 0;
}

const MAX_NAMESIZE: usize = 1024;

unsafe fn cpio_load(dev: *mut Vnode, super_node_ref: *mut *mut Vnode) -> isize {
    /* allocate the root node */
    let mut rootfs: *mut Vnode = core::ptr::null_mut();
    cpio_root_node(dev, &mut rootfs);

    let mut hdr: CpioHeader = core::mem::zeroed();
    let mut offset = 0;
    let mut size = 0;

    while offset < (*dev).size {
        let mut data_offset = offset;
        vfs_read(dev, data_offset as isize, core::mem::size_of::<CpioHeader>(), &mut hdr as *const _ as *mut u8);

        if hdr.magic != CPIO_BIN_MAGIC {
            panic!("invalid cpio archive\n");
        }

        size = hdr.filesize[0] as usize * 0x10000 + hdr.filesize[1] as usize;
        
        data_offset += core::mem::size_of::<CpioHeader>();
        
        let mut path = [0u8; MAX_NAMESIZE];
        vfs_read(dev, data_offset as isize, hdr.namesize as usize, path.as_mut_ptr());

        if strcmp(path.as_ptr(), b".\0".as_ptr()) == 0 {
            offset += (core::mem::size_of::<CpioHeader>() + ((hdr.namesize as usize + 1)/2*2 + (size+1)/2*2) as usize);
            continue;
        }

        if strcmp(path.as_ptr(), b"TRAILER!!!\0".as_ptr()) == 0 {
            /* end of archive */
            break;
        }

        let mut dir  = core::ptr::null();
        let mut name = core::ptr::null();

        /* TODO implement strrchr */
        for i in (0..hdr.namesize as usize).rev() {
            if path[i] == b'/' {
                path[i] = b'\0';
                name = path.as_ptr().offset((i+1) as isize);
                dir  = path.as_ptr();
                break;
            }
        }

        if name.is_null() {
            name = path.as_ptr();
            dir  = b"/\0".as_ptr();
        }
        
        data_offset += (hdr.namesize + (hdr.namesize % 2)) as usize;

        let mut _node = core::ptr::null_mut(); 
        cpio_new_node(strdup(name), &mut hdr, size, data_offset, dev, &mut _node);

        let parent = cpio_find(rootfs, dir);
        cpio_new_child_node(parent, _node);

        offset += (core::mem::size_of::<CpioHeader>() + ((hdr.namesize as usize + 1)/2*2 + (size+1)/2*2) as usize);
    }

    if !super_node_ref.is_null() {
        *super_node_ref = rootfs;
    }

    return 0;
}

unsafe fn cpio_read(vnode: *mut Vnode, offset: isize, len: usize, buf: *mut u8) -> usize {
    if (offset as usize) >= (*vnode).size {
        return 0;
    }

    let mut len = if len < (*vnode).size - offset as usize { len } else { (*vnode).size - offset as usize };
    let p = (*vnode).p as *mut Cpio;
    let super_node = (*p).super_node;

    return vfs_read(super_node, ((*p).data + offset as usize) as isize, len, buf) as usize;
}

unsafe fn cpio_readdir(node: *mut Vnode, offset: isize, dirent: *mut DirectoryEntry) -> usize {
    let mut offset = offset;

    if offset == 0 {
        strcpy((*dirent).d_name.as_mut_ptr(), b".\0".as_ptr());
        return 1;
    }

    if offset == 1 {
        strcpy((*dirent).d_name.as_mut_ptr(), b"..\0".as_ptr());
        return 1;
    }

    offset -= 2;

    let p = (*node).p as *mut Cpio;

    if (offset as usize) == (*p).count {
        return 0;
    }

    let mut i = 0;
    let dir = (*p).dir;

    let mut node = dir; 
    while !node.is_null() {
        let cpio = (*node).p as *mut Cpio;

        if i == offset {
            (*dirent).d_ino = node as ino_t;
            strcpy((*dirent).d_name.as_mut_ptr(), (*cpio).name);   // FIXME
            break;
        }

        i += 1;
        node = (*cpio).next;
    }

    return (i == offset) as usize;
}

unsafe fn cpio_close(_vnode: *mut Vnode) -> isize {
    return 0;
}

unsafe fn cpio_eof(file: *mut FileDescriptor) -> isize {
    if S_ISDIR!((*(*file).backend.vnode).mode) {
        let cpio = (*(*file).backend.vnode).p as *mut Cpio;
        return ((*file).offset >= (*cpio).count as isize) as isize;
    } else {
        return ((*file).offset >= (*(*file).backend.vnode).size as isize) as isize;
    }
}

unsafe fn cpio_init() -> isize {
    return initramfs_archiver_register(&mut cpiofs);
}

static mut cpiofs: Filesystem = Filesystem {
    name: "cpio",
    nodev: 0,
    _load: Some(cpio_load),
    _init: None,
    _mount: None,

    vops: VnodeOps {
        _read:     Some(cpio_read),
        _readdir:  Some(cpio_readdir),
        _finddir:  Some(cpio_finddir),
        _vget:     Some(cpio_vget),
        _close:    Some(cpio_close),

        _write:    Some(rofs_write),
        _trunc:    Some(rofs_trunc),
        _vmknod:   Some(rofs_vmknod),
        _vunlink:  Some(rofs_vunlink),

        _chmod:    None,
        _chown:    None,
        _ioctl:    None,
        _map:      None,
        _sync:     None,
        _vsync:    None,
    },
    
    fops: FileOps {
        _open:     Some(posix_file_open),
        _close:    Some(posix_file_close),
        _read:     Some(posix_file_read),
        _write:    Some(posix_file_write),
        _readdir:  Some(posix_file_readdir),
        _lseek:    Some(posix_file_lseek),

        _eof:      Some(cpio_eof),

        _can_read:  None,
        _can_write: None,
        _ioctl:     None,
        _trunc:     None,
    },
};

module_init!(initramfs_cpio, Some(cpio_init), None);
