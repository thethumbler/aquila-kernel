use prelude::*;

use fs::*;
use fs::posix::*;
use fs::pseudofs::*;
use mm::*;
use kern::time::*;


use crate::{malloc_declare};

malloc_declare!(M_VNODE);

unsafe fn tmpfs_vget(super_node: *mut Vnode, ino: ino_t, vnode: *mut *mut Vnode) -> isize {
    /* vnode is always present in memory */
    let node = ino as usize as *mut Vnode;
    if !vnode.is_null() {
        *vnode = node;
    }

    return 0;
}

unsafe fn tmpfs_close(vnode: *mut Vnode) -> isize {
    /* Inode is always present in memory */

    if ((*vnode).refcnt == 0 && (*vnode).nlink == 0) {
        /* vnode is no longer referenced */
        kfree((*vnode).p);
        pseudofs_close(vnode);
    }

    return 0;
}

unsafe fn tmpfs_read(node: *mut Vnode, offset: off_t, size: size_t, buf: *mut u8) -> usize {
    //printk("tmpfs_read(vnode=%p, offset=%d, size=%d, buf=%p)\n", node, offset, size, buf);

    if (*node).size == 0 {
        return 0;
    }

    let r = min!((*node).size - offset as usize, size);
    memcpy(buf, (*node).p.offset(offset), r);

    return r;
}

unsafe fn tmpfs_write(node: *mut Vnode, offset: off_t, size: size_t, buf: *mut u8) -> usize {
    if (*node).size == 0 {
        let sz = offset as usize + size;
        (*node).p = Buffer::new(sz).leak();
        (*node).size = sz;
    }

    if offset as usize + size > (*node).size {
        /* reallocate */
        let sz = offset as usize + size;
        let new = Buffer::new(sz).leak();

        memcpy(new, (*node).p, (*node).size);
        kfree((*node).p);

        (*node).p = new;
        (*node).size = sz;
    }

    memcpy((*node).p.offset(offset), buf, size);
    return size;
}

unsafe fn tmpfs_trunc(vnode: *mut Vnode, len: off_t) -> isize {
    if vnode.is_null() {
        return -EINVAL;
    }

    if len as usize == (*vnode).size {
        return 0;
    }

    if len == 0 {
        kfree((*vnode).p);
        (*vnode).size = 0;
        return 0;
    }

    let sz = min!(len as usize, (*vnode).size);
    let buf = Buffer::new(len as usize).leak();

    if buf.is_null() {
        panic!("failed to allocate buffer");
    }

    memcpy(buf, (*vnode).p, sz);

    if len as usize > (*vnode).size {
        core::ptr::write_bytes(buf.offset((*vnode).size as isize), 0, len as usize - (*vnode).size);
    }

    kfree((*vnode).p);
    (*vnode).p    = buf;
    (*vnode).size = len as usize;

    return 0;
}

unsafe fn tmpfs_chmod(vnode: *mut Vnode, mode: mode_t) -> isize {
    //printk("vnod->mode = %d\n", mode);
    (*vnode).mode = ((*vnode).mode & !0777) | mode;
    return 0;
}

unsafe fn tmpfs_chown(vnode: *mut Vnode, uid: uid_t, gid: gid_t) -> isize {
    (*vnode).uid = uid;
    (*vnode).gid = gid;
    return 0;
}

/* ================ File Operations ================ */

unsafe fn tmpfs_file_can_read(file: *mut FileDescriptor, size: size_t) -> isize {
    if (*file).offset as usize + size < (*(*file).backend.vnode).size {
        return 1;
    }

    return 0;
}

unsafe fn tmpfs_file_can_write(file: *mut FileDescriptor, size: size_t) -> isize {
    /* TODO impose limit */
    return 1;
}

unsafe fn tmpfs_file_eof(file: *mut FileDescriptor) -> isize {
    return ((*file).offset == (*(*file).backend.vnode).size as isize) as isize;
}

unsafe fn tmpfs_init() -> isize {
    return vfs_install(&mut TMPFS);
}

unsafe fn tmpfs_mount(dir: *const u8, flags: isize, data: *mut u8) -> isize {
    /* Initalize new */
    let mut tmpfs_root = kmalloc(core::mem::size_of::<Vnode>(), &M_VNODE, M_ZERO) as *mut Vnode;

    if tmpfs_root.is_null() {
        return -ENOMEM;
    }

    let mut mode: mode_t = 0o777;

    struct MountData {
        dev: *mut u8,
        opt: *mut u8,
    };

    let mdata: *mut MountData = data as *const _ as *mut MountData;

    if !(*mdata).opt.is_null() {
        let tokens = tokenize((*mdata).opt, b',');
        let mut token_p = tokens; 

        while !(*token_p).is_null() {
            let token = *token_p;

            if strncmp(token, b"mode=\0".as_ptr(), 5) == 0 {    /* ??? */
                let mut t = token.offset(5);
                mode = 0;
                while *t != b'0' {
                    mode <<= 3;
                    mode |= (*t - b'0') as mode_t;

                    t = t.offset(1);
                }
            }

            token_p = token_p.offset(1);
        }
    }

    (*tmpfs_root).ino    = tmpfs_root as ino_t;
    (*tmpfs_root).mode   = S_IFDIR | mode as mode_t;
    (*tmpfs_root).size   = 0;
    (*tmpfs_root).nlink  = 2;
    (*tmpfs_root).fs     = &TMPFS;
    (*tmpfs_root).p      = core::ptr::null_mut();
    (*tmpfs_root).refcnt = 1;

    let mut ts: TimeSpec = core::mem::uninitialized();
    gettime(&mut ts);

    (*tmpfs_root).ctime = ts;
    (*tmpfs_root).atime = ts;
    (*tmpfs_root).mtime = ts;

    vfs_bind(dir, tmpfs_root);

    return 0;
}

pub static mut TMPFS: Filesystem = Filesystem {
    name:    "tmpfs",
    nodev:   1,

    _init:    Some(tmpfs_init),
    _load:    None,
    _mount:   Some(tmpfs_mount),

    vops: VnodeOps {
        _read:     Some(tmpfs_read),
        _write:    Some(tmpfs_write),
        _close:    Some(tmpfs_close),
        _trunc:    Some(tmpfs_trunc),
        _chmod:    Some(tmpfs_chmod),
        _chown:    Some(tmpfs_chown),

        _readdir:  Some(pseudofs_readdir),
        _finddir:  Some(pseudofs_finddir),

        _vmknod:   Some(pseudofs_vmknod),
        _vunlink:  Some(pseudofs_vunlink),
        _vget:     Some(tmpfs_vget),

        _ioctl: None,
        _map:   None,
        _sync:  None,
        _vsync: None,
    },
    
    fops: FileOps {
        _open:     Some(posix_file_open),
        _close:    Some(posix_file_close),
        _read:     Some(posix_file_read),
        _write:    Some(posix_file_write),
        _ioctl:    Some(posix_file_ioctl),
        _lseek:    Some(posix_file_lseek),
        _readdir:  Some(posix_file_readdir),
        _trunc:    Some(posix_file_trunc),

        _can_read:   Some(tmpfs_file_can_read),
        _can_write:  Some(tmpfs_file_can_write),
        _eof:        Some(tmpfs_file_eof),
    },
};

module_init!(tmpfs, Some(tmpfs_init), None);
