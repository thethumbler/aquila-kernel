use prelude::*;

use fs::*;
use mm::*;

use crate::{malloc_define};

/* cacehed block */
pub struct Block {
    off: off_t,
    data: *mut u8,
    dirty: isize,
}

#[repr(C)]
pub struct BlockCache {
    pub hashmap: *mut HashMap<off_t, *mut Block>,
}

malloc_define!(M_BCACHE, "bcache\0", "block cache structure\0");
malloc_define!(M_CACHE_BLOCK, "cache block\0", "cached block structure\0");

//unsafe fn bcache_eq(a: *mut u8, b: *mut u8) -> isize {
//    let a = a as *mut Block;
//    let b = b as *mut off_t;
//
//    return ((*a).off == *b) as isize;
//}

pub unsafe fn bcache_init(bcache: *mut BlockCache) {
    (*bcache).hashmap = HashMap::alloc();
}

pub unsafe fn bcache_insert(bcache: *mut BlockCache, off: off_t, data: *mut u8) -> isize {
    if bcache.is_null() {
        return -EINVAL;
    }

    if (*bcache).hashmap.is_null() {
        bcache_init(bcache);
    }

    let mut block = kmalloc(core::mem::size_of::<Block>(), &M_CACHE_BLOCK, M_ZERO) as *mut Block;
    if block.is_null() {
        return -ENOMEM;
    }

    (*block).off = off;
    (*block).data = data;
    (*block).dirty = 0;

    return (*(*bcache).hashmap).insert(&off, block);
}

pub unsafe fn bcache_remove(bcache: *mut BlockCache, off: off_t) -> isize {
    /*
    if bcache.is_null() || (*bcache).hashmap.is_null() {
        return -EINVAL;
    }

    let node = (*(*bcache).hashmap).lookup(&off);

    if node.is_some() {
        let node = node.unwrap();
        let block = (*node).value;
        (*node).value = core::ptr::null_mut();

        kfree((*node).value as *mut u8);
        (*(*bcache).hashmap).node_remove(&*node);
        return 0;
    }
    */

    return -1;
}

pub unsafe fn bcache_find(bcache: *mut BlockCache, off: off_t) -> *mut u8 {
    if bcache.is_null() || (*bcache).hashmap.is_null() {
        return core::ptr::null_mut();
    }

    let node = (*(*bcache).hashmap).lookup(&off);

    if node.is_some() {
        let node = node.unwrap();
        let block = (*node).value;
        return (*block).data;
    }

    return core::ptr::null_mut();
}

pub unsafe fn bcache_dirty(bcache: *mut BlockCache, off: off_t) {
    if bcache.is_null() || (*bcache).hashmap.is_null() {
        return;
    }

    let node = (*(*bcache).hashmap).lookup(&off);

    if node.is_some() {
        let node = node.unwrap();
        let block = (*node).value;
        (*block).dirty = 1;
    }
}
