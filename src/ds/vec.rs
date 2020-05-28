use prelude::*;

use mm::*;
use crate::{malloc_declare};

malloc_declare!(M_BUFFER);

#[derive(Debug)]
pub struct Vec<T> {
    raw: *mut T,
    cap: usize,
    len: usize,
}

impl<T: Copy> Vec<T> {
    pub const fn new() -> Self {
        Vec {
            raw: core::ptr::null_mut(),
            cap: 0,
            len: 0,
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        Vec {
            raw: unsafe { kmalloc(n * core::mem::size_of::<T>(), &M_BUFFER, M_ZERO) as *mut T },
            cap: n,
            len: 0,
        }
    }

    pub fn insert(&mut self, t: T) {
        if self.len < self.cap {
            unsafe { *self.raw.offset(self.len as isize) = t; }
            self.len += 1;
        } else {
            /* TODO */
        }
    }
}

use core::fmt;
impl<T: fmt::Display> fmt::Display for Vec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            if self.len > 0 {
                fmt::write(f, core::format_args!("["))?;

                for i in 0..self.len-1 {
                    core::fmt::write(f, core::format_args!("{}, ", *self.raw.offset(i as isize)))?;
                }

                core::fmt::write(f, core::format_args!("{}]", *self.raw.offset((self.len-1) as isize)))
            } else {
                core::fmt::write(f, core::format_args!("[]"))
            }
        }
    }
}

impl<T> core::ops::Index<usize> for Vec<T> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        unsafe { &*self.raw.offset(i as isize) }
    }
}

impl<T> core::ops::IndexMut<usize> for Vec<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        unsafe { &mut *self.raw.offset(i as isize) }
    }
}
