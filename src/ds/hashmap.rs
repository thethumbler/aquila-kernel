use prelude::*;

use mm::*;
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

    pub fn hash<K: Hash>(key: &K) -> u64 {
        let mut hasher = Self::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}


use core::marker::PhantomData;

use crate::malloc_define;
use crate::print;

malloc_define!(M_HASHMAP, "hashmap\0", "hashmap structure\0");
malloc_define!(M_HASHMAP_NODE, "hashmap-node\0", "hashmap node structure\0");

pub type hash_t = usize;

const HASHMAP_DEFAULT: usize = 20;

/** hashmap */
pub struct HashMap<K, V> {
    count: usize,
    buckets: Vec<Queue<*mut HashMapNode<K, V>>>,

    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

/** hashmap node */
pub struct HashMapNode<K, V> {
    pub hash: u64,
    pub key: K,
    pub value: V,
    pub qnode: *mut QueueNode<*mut HashMapNode<K, V>>,
}

impl<K: Copy + Eq + Hash, V> HashMapNode<K, V> {
    pub fn new(key: &K, value: V) -> Self {
        HashMapNode {
            hash: DummyHasher::hash(key),
            key: *key,
            value,
            qnode: core::ptr::null_mut(),
        }
    }

    pub fn alloc(value: Self) -> Box<Self> {
        Box::new_tagged(&M_HASHMAP_NODE, value)
    }
}

// TODO use chain iterator
pub struct HashMapIterator<'a, K, V> {
    idx: usize,
    iters: Vec<QueueIterator<'a, *mut HashMapNode<K, V>>>,
    phantom: PhantomData<&'a HashMapNode<K, V>>,
}

impl<K: Copy + Eq + Hash, V> HashMap<K, V> {
    fn bucket_index(&self, hash: u64) -> usize {
        (hash as usize) % self.buckets.len()
    }

    /* XXX */
    pub fn empty() -> HashMap<K, V> {
        HashMap {
            count: 0,
            buckets: Vec::new(),
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        }
    }

    pub fn new(buckets_count: usize) -> HashMap<K, V> {
        let buckets_count = if buckets_count == 0 { HASHMAP_DEFAULT } else { buckets_count };
        let mut buckets = Vec::new();
        buckets.resize(buckets_count, Queue::new());

        HashMap {
            count: 0,
            buckets,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        }
    }

    pub fn alloc(value: Self) -> Box<Self> {
        Box::new_tagged(&M_HASHMAP, value)
    }

    /** insert a new element into a hashmap */
    pub fn insert(&mut self, key: &K, value: V) -> isize {
        let node = Box::leak(HashMapNode::alloc(HashMapNode::new(key, value)));
        let idx = self.bucket_index(node.hash);

        node.qnode = self.buckets[idx].enqueue(node);
        self.count += 1;

        return 0;
    }

    /** lookup for an element in the hashmap using the the key */
    pub fn lookup(&self, key: &K) -> Option<&HashMapNode<K, V>> {
        unsafe {
            if self.buckets.len() == 0 || self.count == 0 {
                return None;
            }

            let hash = DummyHasher::hash(key);
            let idx = self.bucket_index(hash);
            let queue = &self.buckets[idx];
            
            for qnode in queue.iter() {
                let hnode = (*qnode).value;

                if hnode.is_null() {
                    panic!("what?");
                }

                if !hnode.is_null() && (*hnode).hash == hash && *key == (*hnode).key {
                    return Some(&*hnode);
                }
            }

            return None;
        }
    }

    /** remove an element from the hashmap given the hashmap node */
    pub fn node_remove(&mut self, node: &HashMapNode<K, V>) {
        unsafe {
            if self.buckets.len() == 0 || node.qnode.is_null() {
                return;
            }

            let idx   = self.bucket_index((*node).hash);
            let queue = &mut self.buckets[idx];
            let node  = &mut *(node as *const _ as *mut HashMapNode<K, V>);

            (*queue).node_remove(node.qnode);
            Box::from_raw(node);

            self.count -= 1;
        }
    }

    /** free all resources associated with a hashmap */
    pub fn free(mut self) {
        unsafe {
            for i in 0..self.buckets.len() {
                let queue = &mut self.buckets[i];

                let mut node = (*queue).dequeue();

                while !node.is_none() {
                    kfree(node.unwrap() as *mut u8);
                    node = (*queue).dequeue();
                }
            }
        }
    }

    pub fn iter<'a>(&'a self) -> HashMapIterator<K, V> {
        HashMapIterator {
            idx: 0,
            iters: self.buckets.iter().map(|q| q.iter()).collect(),
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for HashMapIterator<'a, K, V> {
    type Item = &'a HashMapNode<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.iters.len() {
            match self.iters[self.idx].next() {
                Some(qnode) => unsafe { Some(&*qnode.value) },
                None => {
                    self.idx += 1;
                    self.next()
                },
            }
        } else {
            None
        }
    }
}
