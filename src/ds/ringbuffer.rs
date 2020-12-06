use prelude::*;

use mm::*;
use ds::buffer::Buffer;
use crate::{malloc_define, malloc_declare};

malloc_define!(M_RINGBUFFER, "ring-buffer\0", "ringbuffer structure\0");

macro_rules! ring_index {
    ($ring:expr, $i:expr) => {
        (($i) % ((*$ring).size()))
    }
}

pub struct RingBuffer {
    buf: Buffer,
    head: usize,
    tail: usize,
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            buf: Buffer::new(size),
            head: 0,
            tail: 0,
        }
    }

    pub fn alloc(val: RingBuffer) -> Box<RingBuffer> {
        Box::new_tagged(&M_RINGBUFFER, val)
    }

    pub fn size(&self) -> usize {
        self.buf.size()
    }

    pub fn available(&self) -> usize {
        if self.tail >= self.head {
            return self.tail - self.head;
        }

        return self.tail + self.size() - self.head;
    }

    pub fn read(&mut self, n: usize, buf: *mut u8) -> usize {
        let size = n;
        let mut n = n;
        let mut buf = buf;

        while n > 0 {
            if self.head == self.tail {
                /* ring is empty */
                break;
            }

            if self.head == self.size() {
                self.head = 0;
            }

            unsafe { *buf = self.buf[self.head]; }
            self.head += 1;
            buf = unsafe { buf.offset(1) };
            n -= 1;
        }

        return size - n;
    }

    pub fn write(&mut self, n: usize, buf: *mut u8) -> usize {
        let size = n;

        let mut n = n;
        let mut buf = buf;

        while n > 0 {
            if ring_index!(self, self.head) == ring_index!(self, self.tail) + 1 {
                /* ring is full */
                break;
            }

            if self.tail == self.size() {
                self.tail = 0;
            }
            
            unsafe { self.buf[self.tail] = *buf; }
            self.tail += 1;
            buf = unsafe { buf.offset(1) };
            n -= 1;
        }

        return size - n;
    }

    pub fn peek(&self, off: off_t, n: usize, buf: *mut u8) -> usize {
        let size = n;

        let mut head = self.head + off as usize;
        let mut n = n;
        let mut buf = buf;

        if self.head < self.tail && head > self.tail {
            return 0;
        }

        while n > 0 {
            if head == self.size() {
                head = 0;
            }

            if head == self.tail { /* Ring is empty */
                break;
            }

            unsafe { *buf = self.buf[head]; }
            head += 1;
            buf = unsafe { buf.offset(1) };
            n -= 1;
        }

        return size - n;
    }

    pub fn write_overwrite(&mut self, n: usize, buf: *mut u8) -> usize {
        let size = n;

        let mut n = n;
        let mut buf = buf;

        while n > 0 {
            if ring_index!(self, self.head) == ring_index!(self, self.tail) + 1 {
                /* move head to match */
                self.head = ring_index!(self, self.head) + 1;
            }

            if self.tail == self.size() {
                self.tail = 0;
            }
            
            unsafe { self.buf[self.tail] = *buf; }
            self.tail += 1;
            buf = unsafe { buf.offset(1) };
            n -= 1;
        }

        return size - n;
    }

}