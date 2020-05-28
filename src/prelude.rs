extern crate alloc;

pub use include::core::types::*;
pub use include::bits::errno::*;
pub use ds::*;

pub use mm::kvmem::*;
pub use kern::string::*;
pub use alloc::boxed::Box;
pub use alloc::rc::Rc;
pub use crate::{print, malloc_define, malloc_declare};

// XXX
pub use include::mm::kvmem::MallocType;
pub use include::mm::kvmem::M_ZERO;

//pub struct AllocTag {
//    pub name: *const u8,
//    pub desc: *const u8,
//    pub nr: usize,
//    pub total: usize,
//    pub qnode: *mut Qnode,
//}

//use include::mm::kvmem::MallocType;
//unsafe impl Sync for AllocTag {}

pub trait TaggedAllocator<T> {
    fn new_tagged(tag: &MallocType, obj: T) -> Box<T> {
        Box::new(obj)
    }

    fn new_uninit_tagged(tag: &MallocType) -> Box<core::mem::MaybeUninit<T>> {
        Box::new_uninit()
    }

    fn new_zeroed_tagged(tag: &MallocType) -> Box<core::mem::MaybeUninit<T>> {
        Box::new_zeroed()
    }
}

impl<T> TaggedAllocator<T> for Box<T> {}
