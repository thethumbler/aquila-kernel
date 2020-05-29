extern crate alloc;

pub use kern::types::*;
pub use bits::errno::*;
pub use ds::*;

pub use mm::kvmem::*;
pub use kern::types::*;
pub use kern::string::*;
pub use kern::module::*;
pub use alloc::boxed::Box;
pub use alloc::rc::Rc;
pub use kern::print::cstr;
pub use crate::{print};

// XXX
pub use mm::kvmem::MallocType;
pub use mm::kvmem::M_ZERO;

pub trait TaggedAllocator<T> {
    fn new_tagged(tag: &MallocType, obj: T) -> Box<T> {
        Box::new(obj)
    }

    fn new_uninit_tagged(tag: &MallocType) -> Box<core::mem::MaybeUninit<T>> {
        Box::new_uninit()
    }

    fn new_zeroed_tagged(tag: &MallocType) -> Box<core::mem::MaybeUninit<T>> {
        unsafe {
            let ptr = kmalloc(core::mem::size_of::<T>(), tag, M_ZERO) as *mut core::mem::MaybeUninit<T>;
            if ptr.is_null() {
                panic!("allocation failed");
            }

            Box::from_raw(ptr)
        }
    }
}

impl<T> TaggedAllocator<T> for Box<T> {}

pub macro max {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    }
}

pub macro min {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    }
}
