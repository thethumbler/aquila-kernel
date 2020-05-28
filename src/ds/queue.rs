use prelude::*;

use core::marker::Sync;
use crate::include::mm::kvmem::*;
use crate::malloc_define;

use crate::print;

malloc_define!(M_QUEUE, "queue\0", "queue structure\0");
malloc_define!(M_QNODE, "queue-node\0", "queue node structure\0");

#[derive(Debug)]
pub struct QueueNode<T> {
    pub value: *mut T,
    pub prev: *mut QueueNode<T>,
    pub next: *mut QueueNode<T>,

    _phantom: core::marker::PhantomData<T>,
}

unsafe impl<T> Sync for QueueNode<T> {}

#[derive(Copy, Clone, Debug)]
pub struct Queue<T> {
    pub count: usize,
    pub head: *mut QueueNode<T>,
    pub tail: *mut QueueNode<T>,
    pub flags: usize,

    _phantom: core::marker::PhantomData<T>,
}

unsafe impl<T: Sync> Sync for Queue<T> {}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self {
            count: 0,
            head: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
            flags: 0,
            _phantom: core::marker::PhantomData,
        }
    }
}

pub struct QueueIterator<'a, T> {
    cur: *mut QueueNode<T>,
    phantom: core::marker::PhantomData<&'a QueueNode<T>>
}

impl<'a, T> Iterator for QueueIterator<'a, T> {
    type Item = &'a QueueNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let node = self.cur;

            if node.is_null() {
                return None
            }

            self.cur = (*node).next;

            Some(&*node)
        }
    }
}

impl<T> Queue<T> {
    /** create a new statically allocated queue */
    pub const fn empty() -> Self {
        Self {
            count: 0,
            head: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
            flags: 0,
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn new() -> *mut Queue<T> {
        unsafe {
            let queue = kmalloc(core::mem::size_of::<Queue<T>>(), &M_QUEUE, M_ZERO) as *mut Queue<T>;
            
            if queue.is_null() {
                return core::ptr::null_mut();
            }

            return queue;
        }
    }

    pub unsafe fn enqueue(&mut self, value: *mut T) -> *mut QueueNode<T> {
        let mut node = kmalloc(core::mem::size_of::<QueueNode<T>>(), &M_QNODE, M_ZERO) as *mut QueueNode<T>;
        if node.is_null() {
            return core::ptr::null_mut();
        }

        (*node).value = value;

        if self.count == 0 {
            /* queue is not initalized */
            self.head = node;
            self.tail = node;
        } else {
            (*node).prev = self.tail;
            (*self.tail).next = node;
            self.tail = node;
        }

        self.count += 1;

        return node;
    }

    pub unsafe fn dequeue(&mut self) -> *mut T {
        if self.count == 0 {
            return core::ptr::null_mut();
        }

        self.count -= 1;

        let head = self.head;

        self.head = (*head).next;

        if !self.head.is_null() {
            (*self.head).prev = core::ptr::null_mut();
        }

        if head == self.tail {
            self.tail = core::ptr::null_mut();
        }

        let value = (*head).value;

        kfree(head as *mut u8);

        return value;
    }

    pub unsafe fn remove(&mut self, value: *mut T) {
        if self.count == 0 {
            return;
        }

        let mut qnode = self.head;

        while !qnode.is_null() {
            if (*qnode).value == value {
                if (*qnode).prev.is_null() {
                    /* head */
                    self.dequeue();
                } else if (*qnode).next.is_null() {
                    /* tail */
                    self.count -= 1;
                    self.tail = (*self.tail).prev;
                    (*self.tail).next = core::ptr::null_mut();
                    kfree(qnode as *mut u8);
                } else {
                    self.count -= 1;
                    (*(*qnode).prev).next = (*qnode).next;
                    (*(*qnode).next).prev = (*qnode).prev;
                    kfree(qnode as *mut u8);
                }

                break;
            }

            qnode = (*qnode).next;
        }
    }

    pub unsafe fn node_remove(&mut self, qnode: *mut QueueNode<T>) {
        if self.count == 0 || qnode.is_null() {
            return;
        }

        if !(*qnode).prev.is_null() {
            (*(*qnode).prev).next = (*qnode).next;
        }

        if !(*qnode).next.is_null() {
            (*(*qnode).next).prev = (*qnode).prev;
        }

        if self.head == qnode {
            self.head = (*qnode).next;
        }

        if self.tail == qnode {
            self.tail = (*qnode).prev;
        }

        self.count -= 1;

        kfree(qnode as *mut u8);

        return;
    }

    pub fn iter<'a>(&'a self) -> QueueIterator<'a, T> {
        unsafe {
            QueueIterator {
                cur: (*self).head,
                phantom: core::marker::PhantomData,
            }
        }
    }
}
