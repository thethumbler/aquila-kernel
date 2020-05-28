use prelude::*;

use mm::*;

use crate::kern::print::cstr;
use crate::{page_round, page_align};

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

pub macro malloc_define {
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

pub macro malloc_declare {
    ($type:ident) => {
        extern "C" {
            static $type: MallocType;
        }
    }
}


malloc_define!(M_BUFFER, "buffer\0", "generic buffer\0");

pub static mut MALLOC_TYPES: Queue<*mut MallocType> = Queue::empty();

#[derive(Copy, Clone)]
pub struct ObjectNode {
    addr: usize, /* Offseting (1GiB), 4-bytes aligned objects */
    free: bool,  /* Free or not flag */
    size: usize, /* Size of one object can be up to 256MiB */
    next: usize, /* Index of the next node */
    objtype: *mut MallocType,
}

impl ObjectNode {
    const fn empty() -> Self {
        Self {
            addr: 0,
            free: false,
            size: 0,
            next: 0,
            objtype: core::ptr::null_mut(),
        }
    }
}

const ARCH_KVMEM_BASE: usize = 0xD0000000;
const ARCH_KVMEM_NODES_SIZE: usize = 0x00100000;

#[no_mangle]
static mut kvmem_used: usize = 0;

#[no_mangle]
static mut kvmem_obj_cnt: usize = 0;

const KVMEM_BASE: usize = ARCH_KVMEM_BASE;

macro_rules! node_addr {
    ($node:expr) => {
        (KVMEM_BASE + (($node).addr) * 4)
    }
}

macro_rules! node_size {
    ($node:expr) => {
        ((($node).size) * 4)
    }
}

const LAST_NODE_INDEX: usize = 100000;
const MAX_NODE_SIZE: usize   = (1 << 26) - 1;

pub static mut NODES: [ObjectNode; LAST_NODE_INDEX] = [ObjectNode::empty(); LAST_NODE_INDEX];

pub unsafe fn kvmem_setup() {
    print!("mm: setting up kernel allocator (nodes={:p}, size={:#x})\n", &NODES, core::mem::size_of_val(&NODES));

    /* setting up initial node */
    NODES[0].addr = 0;
    NODES[0].free = true;
    NODES[0].size = (-1isize) as usize;
    NODES[0].next = LAST_NODE_INDEX;

    /* we have to set qnode to an arbitrary value since
     * enqueue will use kmalloc which would try to enqueue
     * M_QNODE type if qnode == NULL, getting us in an
     * infinite loop
     */
    let m_qnode = &M_QNODE as *const _ as *mut MallocType;
    core::ptr::write_volatile(&mut (*m_qnode).qnode, 0xDEADBEEF as *mut QueueNode<*mut MallocType>);
    (*m_qnode).qnode = MALLOC_TYPES.enqueue(m_qnode);
}

static mut FIRST_FREE_NODE: usize = 0;
unsafe fn get_node() -> usize {
    for i in FIRST_FREE_NODE..LAST_NODE_INDEX {
        if NODES[i].size == 0 {
            return i;
        }
    }

    panic!("cannot find an unused node");
}

unsafe fn release_node(i: usize) {
    if !NODES[i].objtype.is_null() {
        (* NODES[i].objtype).nr -= 1;
    }

    //memset(&nodes[i], 0, sizeof(struct kvmem_node));

    NODES[i].size = 0;
    NODES[i].free = true;
}

unsafe fn get_first_fit_free_node(size: usize) -> usize {
    let mut i = FIRST_FREE_NODE;

    while !NODES[i].free || NODES[i].size < size {
        if NODES[i].next == LAST_NODE_INDEX {
            panic!("cannot find a free node");
        }

        i = NODES[i].next as usize;
    }

    return i;
}

pub unsafe fn kmalloc(size: usize, objtype: *const MallocType, flags: usize) -> *mut u8 {
    //printk(b"kmalloc(size: %d, type: %p, flags: 0x%x)\n\0".as_ptr(), size, objtype, flags);

    let objtype = objtype as *mut MallocType;

    /* round size to 4-byte units */
    let size = (size + 3)/4;

    /* look for a first fit free node */
    let i = get_first_fit_free_node(size);

    //printk(b"allocated node %d\n\0".as_ptr(), i);

    /* mark it as used */
    NODES[i].free = false;

    /* split the node if necessary */
    if NODES[i].size > size {
        let n = get_node();

        NODES[n].addr = NODES[i].addr + size;
        NODES[n].free = true;
        NODES[n].size = NODES[i].size - size;
        NODES[n].next = NODES[i].next;

        NODES[i].next = n;
        NODES[i].size = size;
    }

    NODES[i].objtype = objtype;
    (*objtype).nr += 1;

    (*objtype).total += node_size!(NODES[i]) as usize;

    kvmem_used += node_size!(NODES[i]) as usize;
    kvmem_obj_cnt += 1;

    let map_base = page_align!(node_addr!(NODES[i]));
    let map_end  = page_round!(node_addr!(NODES[i]) + node_size!(NODES[i]));
    let map_size = (map_end - map_base)/PAGE_SIZE;

    if map_size > 0 {
        let mut vm_entry = VmEntry {
            paddr: 0,
            base: map_base,
            size: (map_size * PAGE_SIZE) as usize,
            flags: VM_KRW,
            off: 0,
            qnode: core::ptr::null_mut(),
            vm_anon: core::ptr::null_mut(),
            vm_object: core::ptr::null_mut(),
        };

        vm_map(&mut kvm_space, &mut vm_entry);
    }

    if (*objtype).qnode.is_null() {
        (*objtype).qnode = MALLOC_TYPES.enqueue(objtype);
    }

    let obj = node_addr!(NODES[i]);

    if (flags as usize & M_ZERO) != 0 {
        //memset(obj, 0, size * 4);
        core::ptr::write_bytes(obj as *mut u8, 0, size * 4);
    }

    return obj as *const u8 as *mut u8;
}

pub unsafe fn kfree(ptr: *mut u8) {
    //printk("kfree(%p)\n", _ptr);
    //uintptr_t ptr = (uintptr_t) _ptr;

    let ptr = ptr as usize;

    if ptr < KVMEM_BASE as usize {
        /* that's not even allocatable */
        return;
    }

    /* look for the node containing _ptr -- merge sequential free nodes */
    let mut cur_node = 0;
    let mut prev_node = 0;

    while ptr != node_addr!(NODES[cur_node]) {
        /* check if current and previous node are free */
        if cur_node != 0 && NODES[cur_node].free && NODES[prev_node].free {
            /* check for overflow */
            if NODES[cur_node].size + NODES[prev_node].size <= MAX_NODE_SIZE {
                NODES[prev_node].size += NODES[cur_node].size;
                NODES[prev_node].next  = NODES[cur_node].next;
                release_node(cur_node);
                cur_node = NODES[prev_node].next;
                continue;
            }
        }

        prev_node = cur_node;
        cur_node = NODES[cur_node].next;

        if cur_node == LAST_NODE_INDEX {
            /* trying to free unallocated node */
            return;
        }
    }

    if NODES[cur_node].free {
        /* node is already free, dangling pointer? */
        print!("double free detected at {:p}\n", ptr as *const u8);

        if !NODES[cur_node].objtype.is_null() {
            print!("object type: {}\n", cstr((*NODES[cur_node].objtype).name));
        }

        panic!("double free");
    }

    /* first we mark our node as free */
    NODES[cur_node].free = true;

    if !NODES[cur_node].objtype.is_null() {
        (*NODES[cur_node].objtype).total -= node_size!(NODES[cur_node]);
        (*NODES[cur_node].objtype).nr -= 1;

        NODES[cur_node].objtype = core::ptr::null_mut();
    }

    kvmem_used -= node_size!(NODES[cur_node]);
    kvmem_obj_cnt -= 1;

    /* now we merge all free nodes ahead -- except the last node */
    while NODES[cur_node].next < LAST_NODE_INDEX && NODES[cur_node].free {
        /* check if current and previous node are free */
        if cur_node != 0 && NODES[cur_node].free && NODES[prev_node].free {
            /* check for overflow */
            if NODES[cur_node].size + NODES[prev_node].size <= MAX_NODE_SIZE {
                NODES[prev_node].size += NODES[cur_node].size;
                NODES[prev_node].next  = NODES[cur_node].next;
                release_node(cur_node);
                cur_node = NODES[prev_node].next;
                continue;
            }
        }

        prev_node = cur_node;
        cur_node = NODES[cur_node].next;
    }

    cur_node = 0;
    while NODES[cur_node].next < LAST_NODE_INDEX {
        if NODES[cur_node].free {
            //struct vm_entry vm_entry = {0};

            //vm_entry.paddr = 0;
            //vm_entry.base  = NODE_ADDR(nodes[cur_node]);
            //vm_entry.size  = NODE_SIZE(nodes[cur_node]);
            //vm_entry.flags = VM_KRW;

            let vaddr = node_addr!(NODES[cur_node]);
            let size  = node_size!(NODES[cur_node]);

            //vm_unmap(&kvm_space, &vm_entry);

            /* XXX */
            //if (size < PAGE_SIZE) {
            //    goto next;
            //}

            let mut sva = page_round!(vaddr);
            let eva = page_align!(vaddr + size);

            let mut nr = (eva - sva)/PAGE_SIZE;

            while nr > 0 {
                //let paddr = arch_page_get_mapping(kvm_space.pmap, sva);

                //if paddr != 0 {
                //    //pmap_remove(kvm_space.pmap, sva as u32, sva as u32 + PAGE_SIZE);
                //    //buddy_free(BUDDY_ZONE_NORMAL, paddr, PAGE_SIZE as usize);
                //}

                sva += PAGE_SIZE;
                nr -= 1;
            }
        }

        cur_node = NODES[cur_node].next;
    }
}
