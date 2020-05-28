use prelude::*;
use fs::*;
use mm::*;

use crate::{malloc_define};

malloc_define!(M_VCACHE, "vcache\0", "vnode cache structure\0");

/* vnode cache */
pub struct VirtualNodeCache {
    hashmap: *mut HashMap<ino_t, *mut Vnode>,
}

/*
impl VirtualNodeCache {
    /* initialize vnode caching structure */
    pub fn new() -> Self {
        VirtualNodeCache {
            hashmap: HashMap::alloc(),
        }
    }

    pub fn insert(&mut self, vnode: *mut Vnode) -> isize {
        return self.hashmap.insert(&(*vnode).ino, vnode);
    }

    pub fn remove(&mut self, vnode: *mut Vnode) -> isize {
        if let Some(node) = self.hashmap.lookup(&(*vnode).ino) {
            self.hashmap.node_remove(node);
            return 0;
        }

        return -1;
    }

    pub fn find(&self, ino: ino_t) -> *mut Vnode {
        if let Some(node) = vcache.hashmap.lookup(&ino) {
            return node.value as *mut Vnode;
        }

        return core::ptr::null_mut();
    }
}
*/

/* initialize vnode caching structure */
pub unsafe fn vcache_init(vcache: *mut VirtualNodeCache) {
    (*vcache).hashmap = HashMap::alloc();
}

pub unsafe fn vcache_insert(vcache: *mut VirtualNodeCache, vnode: *mut Vnode) -> isize {
    if vcache.is_null() {
        return -EINVAL;
    }

    if (*vcache).hashmap.is_null() {
        vcache_init(vcache);
    }

    return (*(*vcache).hashmap).insert(&(*vnode).ino, vnode);
}

pub unsafe fn vcache_remove(vcache: *mut VirtualNodeCache, vnode: *mut Vnode) -> isize {
    if vcache.is_null() || (*vcache).hashmap.is_null() {
        return -1;
    }

    let node = (*(*vcache).hashmap).lookup(&(*vnode).ino);

    if node.is_some() {
        let node = node.unwrap();
        (*(*vcache).hashmap).node_remove(node);
        return 0;
    }

    return -1;
}

pub unsafe fn vcache_find(vcache: *mut VirtualNodeCache, ino: ino_t) -> *mut Vnode {
    if vcache.is_null() || (*vcache).hashmap.is_null() {
        return core::ptr::null_mut();
    }

    let node = (*(*vcache).hashmap).lookup(&ino);

    if node.is_some() {
        let node = node.unwrap();
        return (*node).value;
    }

    return core::ptr::null_mut();
}
