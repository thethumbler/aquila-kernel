use prelude::*;

use fs::{self, *};
use fs::posix::*;
use mm::*;
use kern::time::*;
use sys::syscall::file::{FileDescriptor, FileBackend};


use crate::{malloc_declare};

malloc_declare!(M_VNODE);

fn iget(_superblock: &mut Node, ino: ino_t) -> Result<&'static mut Node, Error> {
    /* node is always present in memory */
    // TODO bounds check?
    unsafe {
        let node = ino as usize as *mut Node;
        Ok(&mut *node)
    }
}

fn iput(_superblock: &mut Node, _node: &mut Node) -> Result<(), Error> {
    // do nothing
    Ok(())
}

fn close(node: &Node) -> Result<usize, Error> {
    /* inode is always present in memory */

    if node.refcnt == 0 && node.nlink() == 0 {
        /* vnode is no longer referenced */
        //kfree((*vnode).p);
        unsafe { pseudofs::close(node); }
    }

    return Ok(0);
}

fn read(node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
    unsafe {
        //printk("tmpfs_read(vnode=%p, offset=%d, size=%d, buf=%p)\n", node, offset, size, buf);

        if node.size() == 0 {
            return Ok(0);
        }

        let r = min!(node.size() - offset, size);
        memcpy(buffer, (node.data::<u8>().unwrap() as *mut u8).offset(offset as isize), r);

        return Ok(r);
    }
}

fn write(node: &Node, offset: usize, size: usize, buffer: *mut u8) -> Result<usize, Error> {
    unsafe {
        if node.size() == 0 {
            let sz = offset + size;
            node.set_data(Buffer::new(sz).leak());
            node.set_size(sz);
        }

        if offset + size > node.size() {
            /* reallocate */
            let sz = offset + size;
            let new = Buffer::new(sz).leak();

            memcpy(new, node.data::<u8>().unwrap() as *mut u8, node.size());
            //kfree((*node).p);

            node.set_data(new);
            node.set_size(sz);
        }

        memcpy((node.data::<u8>().unwrap() as *mut u8).offset(offset as isize), buffer, size);

        return Ok(size);
    }
}

fn trunc(node: &Node, len: usize) -> Result<usize, Error> {
    unsafe {
        if len == node.size() {
            return Ok(0);
        }

        if len == 0 {
            //kfree((*vnode).p);
            node.set_size(0);
            return Ok(0);
        }

        let sz = min!(len, node.size());
        let buf = Buffer::new(len).leak();

        if buf.is_null() {
            panic!("failed to allocate buffer");
        }

        memcpy(buf, node.data::<u8>().unwrap() as *mut u8, sz);

        if len > node.size() {
            core::ptr::write_bytes(buf.offset(node.size() as isize), 0, len - node.size());
        }

        //kfree((*vnode).p);
        node.set_data(buf);
        node.set_size(len);

        return Ok(0);
    }
}

fn chmod(node: &Node, mode: mode_t) -> Result<mode_t, Error> {
    let old_mode = node.mode();
    node.set_mode((old_mode & !0o777) | mode);

    Ok(old_mode)
}

fn chown(node: &Node, uid: uid_t, gid: gid_t) -> Result<(uid_t, gid_t), Error> {
    let old = (node.uid(), node.gid());

    node.set_uid(uid);
    node.set_gid(gid);

    Ok(old)
}

/* ================ File Operations ================ */

unsafe fn tmpfs_file_can_read(file: *mut FileDescriptor, size: size_t) -> isize {
    if (*file).offset as usize + size < (*(*file).backend.vnode).size() {
        return 1;
    }

    return 0;
}

unsafe fn tmpfs_file_can_write(file: *mut FileDescriptor, size: size_t) -> isize {
    /* TODO impose limit */
    return 1;
}

unsafe fn tmpfs_file_eof(file: *mut FileDescriptor) -> isize {
    return ((*file).offset == (*(*file).backend.vnode).size() as isize) as isize;
}

fn init() -> Result<(), Error> {
    //fs::install(&mut TMPFS)

    fs::install(Arc::new(Filesystem {
        name:    "tmpfs",
        nodev:   1,

        init:    Some(init),
        mount:   Some(mount),

        read:     Some(read),
        write:    Some(write),
        close:    Some(close),
        trunc:    Some(trunc),
        chmod:    Some(chmod),
        chown:    Some(chown),

        readdir:  Some(pseudofs::readdir),
        finddir:  Some(pseudofs::finddir),

        mknod:    Some(pseudofs::mknod),
        unlink:   Some(pseudofs::unlink),
        iget:     Some(iget),
        iput:     Some(iput),
        
        fops: FileOps {
            _open:     Some(posix_file_open),
            //_close:    Some(posix_file_close),

            _can_read:   Some(tmpfs_file_can_read),
            _can_write:  Some(tmpfs_file_can_write),
            _eof:        Some(tmpfs_file_eof),
        },

        ..Filesystem::none()
    }))
}

fn mount(_fs: Arc<Filesystem>, dir: &str, flags: isize, data: *mut u8) -> Result<(), Error> {
    unsafe {
        let ts = gettime()?;

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

        
        let mut tmpfs_root = Node::none();

        //(*tmpfs_root).ino    = tmpfs_root as ino_t;
        tmpfs_root.set_mode(S_IFDIR | mode as mode_t);
        tmpfs_root.set_size(0);
        tmpfs_root.set_nlink(2);
        tmpfs_root.set_data::<u8>(core::ptr::null_mut());

        tmpfs_root.fs = Some(_fs);
        tmpfs_root.refcnt = 1;

        tmpfs_root.ctime = ts;
        tmpfs_root.atime = ts;
        tmpfs_root.mtime = ts;

        fs::bind(dir, Arc::new(tmpfs_root))
    }
}

module_define!{
    "tmpfs",
    None,
    Some(init),
    None
}
