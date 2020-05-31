use prelude::*;

use core::marker::Sync;
use mm::*;
use crate::malloc_define;

use crate::print;

malloc_define!(M_QUEUE, "queue\0", "queue structure\0");
malloc_define!(M_QNODE, "queue-node\0", "queue node structure\0");

#[repr(C)]
#[derive(Debug)]
pub struct QueueNode<T> {
    pub value: T,
    pub prev: *mut QueueNode<T>,
    pub next: *mut QueueNode<T>,
}

impl<T> QueueNode<T> {
    pub fn new(value: T) -> QueueNode<T> {
        QueueNode {
            value,
            prev: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
        }
    }

    pub fn alloc(val: QueueNode<T>) -> Box<QueueNode<T>> {
        Box::new_tagged(&M_QNODE, val)
    }
}

unsafe impl<T> Sync for QueueNode<T> {}

#[derive(Copy, Clone, Debug)]
pub struct Queue<T> {
    count: usize,
    pub head: *mut QueueNode<T>,
    pub tail: *mut QueueNode<T>,
    pub flags: usize,
}

unsafe impl<T: Sync> Sync for Queue<T> {}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self {
            count: 0,
            head: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
            flags: 0,
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

impl<T: Copy> Queue<T> {
    /** create a new statically allocated queue */
    pub const fn empty() -> Self {
        Self {
            count: 0,
            head: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
            flags: 0,
        }
    }

    pub fn new() -> Queue<T> {
        Queue::empty()
    }

    pub fn alloc(val: Queue<T>) -> Box<Queue<T>> {
        Box::new_tagged(&M_QUEUE, val)
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn enqueue(&mut self, value: T) -> *mut QueueNode<T> {
        unsafe {
            let mut node = Box::leak(QueueNode::alloc(QueueNode::new(value)));

            if self.count == 0 {
                /* queue is not initalized */
                self.head = node;
                self.tail = node;
            } else {
                node.prev = self.tail;
                (*self.tail).next = node;
                self.tail = node;
            }

            self.count += 1;

            return node;
        }
    }

    pub fn enqueue_before(&mut self, qnode: *mut QueueNode<T>, value: T) -> *mut QueueNode<T> {
        unsafe {
            let mut node = Box::leak(QueueNode::alloc(QueueNode::new(value)));

            node.prev  = (*qnode).prev;
            node.next  = qnode;

            if !(*qnode).prev.is_null() {
                (*(*qnode).prev).next = node;
            }

            (*qnode).prev = node;

            if qnode == self.head {
                self.head = node;
            }

            self.count += 1;

            return node;
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        unsafe {
            if self.count == 0 {
                return None
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

            Box::from_raw(head);

            return Some(value);
        }
    }

    pub fn node_remove(&mut self, qnode: *mut QueueNode<T>) {
        unsafe {
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

            Box::from_raw(qnode);

            return;
        }
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

impl<T: Copy + PartialEq> Queue<T> {
    pub fn remove(&mut self, value: T) {
        unsafe {
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
                        Box::from_raw(qnode);
                    } else {
                        self.count -= 1;
                        (*(*qnode).prev).next = (*qnode).next;
                        (*(*qnode).next).prev = (*qnode).prev;
                        Box::from_raw(qnode);
                    }

                    break;
                }

                qnode = (*qnode).next;
            }
        }
    }
}
