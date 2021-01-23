#![no_std]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]
#![feature(alloc_prelude)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(new_uninit)]
#![feature(const_in_array_repeat_expressions)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(map_first_last)]
#![feature(str_split_once)]

#![feature(const_mut_refs)]

#![allow(non_camel_case_types)]
#![allow(unused)]
#![allow(deprecated)]

extern crate alloc;

pub mod prelude;
pub mod panic;

#[macro_use]
pub mod arch;
pub mod kern;
pub mod mm;
pub mod ds;
pub mod sys;
pub mod fs;
pub mod dev;
pub mod net;
pub mod bits;
pub mod boot;

use mm::*;

use alloc::alloc::{GlobalAlloc, Layout, AllocError};
malloc_declare!(M_BUFFER);

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        kmalloc(layout.size(), &M_BUFFER, 0)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        kfree(ptr);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocatiton failure: {:?}\n", layout);
}

#[global_allocator]
static GLOBAL: Allocator = Allocator;

#[lang = "eh_personality"]
pub extern fn rust_eh_personality() {}
