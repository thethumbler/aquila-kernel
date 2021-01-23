use prelude::*;
use fs::{self, *};
use mm::*;
use bits::dirent::*;
use kern::time::*;
use crate::{malloc_define, malloc_declare};
use alloc::collections::btree_map::BTreeMap;

malloc_define!(M_PSEUDOFS_DENT, "pseudofs-dirent\0", "pseudofs directory entry\0");
malloc_declare!(M_VNODE);

type PseudoDirectory = BTreeMap<String, Arc<Node>>;

pub fn mknod(dir: &Node, name: &str, mode: mode_t, dev: dev_t, uio: &UserOp) -> Result<Arc<Node>, Error> {
    unsafe {
        if let Ok(_) = finddir(dir, name) {
            return Err(Error::EEXIST);
        }

        let mut node = Node::none();
        
        //node.ino = &*node as *const _ as ino_t;
        node.set_mode(mode);
        node.set_size(0);
        node.set_uid(uio.uid);
        node.set_gid(uio.gid);
        node.set_nlink(if S_ISDIR!(mode) { 2 } else { 1 });
        node.rdev  = dev;

        let ts = gettime()?;

        node.ctime = ts;
        node.atime = ts;
        node.mtime = ts;

        /* copy filesystem from directory */
        node.fs = dir.fs.as_ref().map(|fs| Arc::clone(&fs));

        let node = Arc::new(node);

        if let None = dir.data::<PseudoDirectory>() {
            dir.set_data(Box::leak(Box::new(PseudoDirectory::new())));
        }

        dir.data::<PseudoDirectory>().unwrap()
            .insert(name.to_owned(), Arc::clone(&node));

        Ok(node)
    }
}

pub fn unlink(node: &Node, name: &str, uio: &UserOp) -> Result<(), Error> {
    unsafe {
        if !node.is_directory() {
            return Err(Error::ENOTDIR);
        }

        node.data::<PseudoDirectory>().unwrap()
            .remove(name)
            .map(|_| Ok(()))
            .unwrap_or(Err(Error::ENOENT))
    }
}

pub fn readdir(dir: &Node, offset: usize) -> Result<(usize, DirectoryEntry), Error> {
    match offset {
        0 => Ok((1, DirectoryEntry::new(0, "."))),
        1 => Ok((1, DirectoryEntry::new(0, ".."))),
        offset => dir.data::<PseudoDirectory>()
            .map(|map| {
                map.iter()
                .nth(offset - 2)
                .map(|(name, node)| unsafe { Ok((1, DirectoryEntry::new((**node).ino, name))) })
                .unwrap_or(Ok((0, DirectoryEntry::none())))
            }).unwrap_or(Ok((0, DirectoryEntry::none())))
    }
}

pub fn finddir(dir: &Node, name: &str) -> Result<DirectoryEntry, Error> {
    unsafe {
        if !dir.is_directory() {
            return Err(Error::ENOTDIR);
        }

        dir.data::<PseudoDirectory>().map(|map| {
            map.get(name)
                .map(|node| Ok(DirectoryEntry::new(&(**node) as *const _ as ino_t, name)))
                .unwrap_or(Err(Error::ENOENT))
        }).unwrap_or(Err(Error::ENOENT))
    }
}

pub fn close(node: &Node) -> isize {
    /* XXX */
    //kfree(inode->name);
    //kfree(vnode as *mut u8);
    return 0;
}
