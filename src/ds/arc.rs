use crate::prelude::*;
use core::sync::atomic::AtomicUsize;
use core::ptr::NonNull;
use core::sync::atomic::Ordering::{Relaxed, SeqCst};
use core::ops::{Deref, DerefMut};

struct ArcInner<T> {
    strong: AtomicUsize,
    data: T,
}

impl<T> ArcInner<T> {
    fn new(data: T) -> Self {
        ArcInner { strong: AtomicUsize::new(1), data }
    }
}

pub struct Arc<T> where T: Alloc {
    ptr: core::ptr::NonNull<ArcInner<T>>
}

impl<T: Alloc> Arc<T> {
    pub fn new(data: T) -> Self {
        let inner = ArcInner::new(data);
        let ptr = Box::leak(Box::new_tagged(<T as Alloc>::alloc_tag(), inner));

        Arc { ptr: NonNull::new(ptr).unwrap() }
    }

    fn inner(&self) -> &ArcInner<T> {
        unsafe { self.ptr.as_ref() }
    }

    fn inner_mut(&mut self) -> &mut ArcInner<T> {
        unsafe { self.ptr.as_mut() }
    }

    fn from_inner(ptr: NonNull<ArcInner<T>>) -> Self {
        Arc { ptr }
    }

    pub fn strong_count(arc: &Self) -> usize {
        arc.inner().strong.load(SeqCst)
    }

    pub fn clone(arc: &Self) -> Self {
        let count = arc.inner().strong.fetch_add(1, Relaxed);
        Arc::from_inner(arc.ptr)
    }
}

impl<T: Alloc> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.inner().strong.fetch_sub(1, Relaxed) == 0 {
            unsafe { Box::from_raw(self.ptr.as_mut()); }
        }
    }
}

impl<T: Alloc> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}

impl<T: Alloc> DerefMut for Arc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner_mut().data
    }
}

//unsafe impl Send for Arc<T>;
//impl<T> TaggedAllocator<T> for Arc<T> where Box<T>: TaggedAllocator<T> {}
