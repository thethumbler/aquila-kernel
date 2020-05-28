use prelude::*;

use core::hash::{Hash, Hasher};
struct DummyHasher {
    state: u64,
}

impl Hasher for DummyHasher {
    fn write(&mut self, data: &[u8]) {
        for b in data {
            self.state += *b as u64;
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

impl DummyHasher {
    fn new() -> Self {
        DummyHasher {
            state: 0
        }
    }
}


use crate::include::core::string::*;
use crate::include::bits::errno::*;

use core::marker::PhantomData;

use crate::include::mm::kvmem::*;
use crate::malloc_define;
use crate::print;

malloc_define!(M_HASHMAP, "hashmap\0", "hashmap structure\0");
malloc_define!(M_HASHMAP_NODE, "hashmap-node\0", "hashmap node structure\0");

pub type hash_t = usize;

const HASHMAP_DEFAULT: usize = 20;

/** hashmap */
#[repr(C)]
pub struct HashMap<K, V> {
    count: usize,
    buckets: *mut Queue<*mut HashMapNode<K, V>>,
    buckets_count: usize,

    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

/** hashmap node */
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HashMapNode<K, V> {
    pub hash: u64,
    pub key: K,
    pub value: V,
    pub qnode: *mut QueueNode<*mut HashMapNode<K, V>>,
}

pub struct HashMapIterator<'a, K, V> {
    idx: usize,
    cur: *mut QueueNode<*mut HashMapNode<K, V>>,
    buckets: *mut Queue<*mut HashMapNode<K, V>>,
    buckets_count: usize,
    phantom: PhantomData<&'a HashMapNode<K, V>>,
}

impl<K, V> HashMap<K, V> 
where K: Copy + Eq + Hash, V: Copy
{
    fn hash(&self, key: &K) -> u64 {
        let mut hasher = DummyHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn bucket_index(&self, hash: u64) -> usize {
        (hash as usize) % self.buckets_count
    }

    /* XXX */
    pub fn empty() -> HashMap<K, V> {
        HashMap {
            count: 0,
            buckets: core::ptr::null_mut(),
            buckets_count: 0,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        }
    }

    pub fn new(buckets_count: usize) -> HashMap<K, V> {
        let buckets_count = if buckets_count == 0 { HASHMAP_DEFAULT } else { buckets_count };

        HashMap {
            count: 0,
            buckets: unsafe { kmalloc(buckets_count * core::mem::size_of::<Queue<*mut HashMapNode<K, V>>>(), &M_QUEUE, M_ZERO) as *mut Queue<*mut HashMapNode<K, V>> },
            buckets_count: buckets_count,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        }
    }

    pub fn alloc() -> *mut HashMap<K, V> {
        //Box::new(&M_HASHMAP, 0, HashMap::new(buckets_count))
        
        unsafe {
            let mut hashmap = kmalloc(core::mem::size_of::<HashMap<K, V>>(), &M_HASHMAP, M_ZERO) as *mut HashMap<K, V>;
            *hashmap = HashMap::new(0);
            //print!("HashMap::alloc -> {:p}\n", hashmap);
            return hashmap;
        }
    }

    /** insert a new element into a hashmap */
    pub fn insert(&mut self, key: &K, value: V) -> isize {
        unsafe {
            //print!("HashMap::insert(self={:p}, key={:p}({}), value={:p})\n", self as *const _ as *const u8, key, core::mem::size_of::<K>(), &value);

            let hash = self.hash(key);
            let node = kmalloc(core::mem::size_of::<HashMapNode<K, V>>(), &M_HASHMAP_NODE, M_ZERO) as *mut HashMapNode<K, V>;

            if node.is_null() {
                return -ENOMEM;
            }

            let idx = self.bucket_index(hash);

            (*node).key   = *key;
            (*node).hash  = hash;
            (*node).value = value;
            (*node).qnode = (*self.buckets.offset(idx as isize)).enqueue(node);

            if (*node).qnode.is_null() {
                //print!(">>>>>>> insert failed\n");
                kfree(node as *mut u8);
                return -ENOMEM;
            }

            self.count += 1;

            return 0;
        }
    }

    /** lookup for an element in the hashmap using the the key */
    pub fn lookup(&self, key: &K) -> Option<&HashMapNode<K, V>> {
        unsafe {
            //print!("HashMap::lookup(self={:p}, key={:p})\n", self as *const _ as *const u8, key as *const _ as *const u8);

            if self.buckets_count == 0 || self.count == 0 {
                //print!("hashmap has no elements\n");
                return None;
            }

            let hash = self.hash(key);
            let idx = self.bucket_index(hash);
            let queue = self.buckets.offset(idx as isize);
            
            for qnode in (*queue).iter() {
                let hnode = (*qnode).value as *mut HashMapNode<K, V>;

                if hnode.is_null() {
                    //print!("how?\n");
                    panic!();
                }

                if !hnode.is_null() && (*hnode).hash == hash && *key == (*hnode).key {
                    return Some(&*hnode);
                }
            }

            return None;
        }
    }

    /** replace an element in the hashmap using the hash and the key */
    pub fn replace(&mut self, key: &K, value: V) -> isize {
        /*
        if hashmap.is_null() || (*hashmap).buckets.is_null() || (*hashmap).buckets_count == 0 || (*hashmap).count == 0 {
            //return -EINVAL;
            return -1;
        }

        let idx = hash % (*hashmap).buckets_count;
        let queue = (*hashmap).buckets.offset(idx as isize);

        let mut qnode = (*queue).head;

        while !qnode.is_null() {
            let hnode = (*qnode).value as *mut HashMapNode;

            if !hnode.is_null() && (*hnode).hash == hash && ((*hashmap).eq)((*hnode).entry, key) != 0 {
                (*hnode).entry = entry;
                return 0;
            }

            qnode = (*qnode).next;
        }

        return hashmap_insert(hashmap, hash, entry);
        */
        0
    }

    /** remove an element from the hashmap given the hashmap node */
    pub fn node_remove(&mut self, node: &HashMapNode<K, V>) {
        unsafe {
            if self.buckets.is_null() || self.buckets_count == 0 || node.qnode.is_null() {
                return;
            }

            let idx   = self.bucket_index((*node).hash);
            let queue = self.buckets.offset(idx as isize);

            (*queue).node_remove((*node).qnode);
            kfree(node as *const _ as *mut u8);

            self.count -= 1;
        }
    }

    /** free all resources associated with a hashmap */
    pub fn free(self) {
        unsafe {
            for i in 0..self.buckets_count {
                let queue = self.buckets.offset(i as isize);

                let mut node = (*queue).dequeue();

                while !node.is_none() {
                    kfree(node.unwrap() as *mut u8);
                    node = (*queue).dequeue();
                }
            }

            kfree(self.buckets as *mut u8);
            //kfree(&self as *const _ as *mut u8);
        }
    }

    pub fn iter(&self) -> HashMapIterator<K, V> {
        HashMapIterator {
            idx: 0,
            buckets: self.buckets,
            buckets_count: self.buckets_count,
            cur: unsafe { (*self.buckets).head },
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for HashMapIterator<'a, K, V> {
    type Item = &'a HashMapNode<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.idx < self.buckets_count {
                if !self.cur.is_null() {
                    let node = (*self.cur).value as *mut HashMapNode<K, V>;
                    self.cur = (*self.cur).next;
                    return Some(&*node);
                } else {
                    self.idx += 1;

                    if self.idx < self.buckets_count {
                        self.cur = (*self.buckets.offset(self.idx as isize)).head;
                        return self.next();
                    } else {
                        return None;
                    }
                }
            }

            None
        }
    }
}
