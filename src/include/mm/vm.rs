use prelude::*;

use crate::fs::vnode::*;

use crate::include::core::types::*;
use crate::include::mm::mm::*;
use crate::include::mm::pmap::*;
use crate::include::mm::kvmem::*;
use crate::{page_align, malloc_declare};

use crate::mm::*;

malloc_declare!(M_VM_ENTRY);

pub use crate::arch::i386::mm::i386::PhysicalMap;

pub type paddr_t = usize;
pub type vaddr_t = usize;

/* vm entry flags */
pub const VM_KR: usize       = 0x0001;        /**< kernel read */
pub const VM_KW: usize       = 0x0002;        /**< kernel write */
pub const VM_KX: usize       = 0x0004;        /**< kernel execute */
pub const VM_UR: usize       = 0x0008;        /**< user read */
pub const VM_UW: usize       = 0x0010;        /**< user write */
pub const VM_UX: usize       = 0x0020;        /**< user execute */
pub const VM_PERM: usize     = 0x003F;        /**< permissions mask */
pub const VM_NOCACHE: usize  = 0x0040;        /**< disable caching */
pub const VM_SHARED: usize   = 0x0080;        /**< shared mapping */
pub const VM_COPY: usize     = 0x0100;        /**< needs copy */

pub const VM_KRW: usize  = (VM_KR|VM_KW);       /**< kernel read/write */
pub const VM_KRX: usize  = (VM_KR|VM_KX);       /**< kernel read/execute */
pub const VM_KWX: usize  = (VM_KW|VM_KX);       /**< kernel write/execute */
pub const VM_KRWX: usize = (VM_KR|VM_KW|VM_KX); /**< kernel read/write/execute */
pub const VM_URW: usize  = (VM_UR|VM_UW);       /**< user read/write */
pub const VM_URX: usize  = (VM_UR|VM_UX);       /**< user read/execute */
pub const VM_UWX: usize  = (VM_UW|VM_UX);       /**< user write/execute */
pub const VM_URWX: usize = (VM_UR|VM_UW|VM_UX); /**< user read/write/execute */

/* object types */
pub const VMOBJ_ZERO: usize = 0x0000;       /**< zero fill */
pub const VMOBJ_FILE: usize = 0x0001;       /**< file backed */

/*
 * \ingroup mm
 * \brief pager
 */
#[repr(C)]
pub struct VmPager {
    /* page in */
    pub page_in: unsafe fn(vm_object: *mut VmObject, off: off_t) -> *mut VmPage,

    /* page out */
    pub page_out: unsafe fn(vm_object: *mut VmObject, off: off_t) -> isize,
}

/* 
 * \ingroup mm
 * \brief physical page
 */
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VmPage {
    /** physical address of the page */
    pub paddr: paddr_t,

    /** the object this page belongs to */
    pub vm_object: *mut VmObject,

    /** offset of page inside the object */
    pub off: off_t,

    /** number of processes referencing this page */
    pub refcnt: usize,
}

unsafe impl Sync for VmPage {}
