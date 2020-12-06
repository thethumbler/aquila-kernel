use prelude::*;

use core::ops::{Deref, DerefMut};
use mm::*;
use crate::malloc_define;

malloc_define!(M_BUFFER, "buffer\0", "generic buffer\0");

// fixed size buffer
pub struct Buffer {
    size: usize,
    data: *mut u8,
}

impl Buffer {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            data: unsafe { kmalloc(size, &M_BUFFER, 0) },
        }
    }

    pub fn alloc(val: Buffer) -> Box<Self> {
        Box::new_tagged(&M_BUFFER, val)
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn leak(self) -> *mut u8 {
        let ret = self.data;
        core::mem::forget(self);

        ret
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { kfree(self.data); }
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.data, self.size) }
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.size) }
    }
}