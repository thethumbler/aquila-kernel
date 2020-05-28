use prelude::*;

#[repr(C)]
pub struct MallocType {
    pub name: *const u8,
    pub desc: *const u8,
    pub nr: usize,
    pub total: usize,
    pub qnode: *mut QueueNode<*mut MallocType>,
}

unsafe impl Sync for MallocType {}

/* malloc flags */
pub const M_ZERO: usize = 0x0001;

#[macro_export]
macro_rules! malloc_define {
    ($type:ident, $name:literal, $desc:literal) => {
        #[no_mangle]
        pub static $type: MallocType = MallocType {
            name: $name.as_ptr(),
            desc: $desc.as_ptr(),
            nr: 0,
            total: 0,
            qnode: core::ptr::null_mut()
        };
    }
}

#[macro_export]
macro_rules! malloc_declare {
    ($type:ident) => {
        extern "C" {
            static $type: MallocType;
        }
    }
}

//extern int debug_kmalloc;
