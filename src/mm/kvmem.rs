use prelude::*;

use mm::vmm::*;
use crate::mm::*;

use mm::vmm::kvm_space;
use crate::include::mm::mm::*;
use crate::include::mm::vm::*;
use crate::include::mm::kvmem::*;
use crate::include::mm::pmap::*;
use crate::include::mm::buddy::*;
use crate::include::core::string::*;
use crate::kern::print::cstr;
use crate::{page_round, page_align, malloc_define, print};

malloc_define!(M_BUFFER, "buffer\0", "generic buffer\0");

pub static mut malloc_types_queue: Queue<MallocType> = Queue::empty();
pub static mut malloc_types: *mut Queue<MallocType> = unsafe { &mut malloc_types_queue };

#[repr(C)]
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

//struct kvmem_node *nodes = (struct kvmem_node *) KVMEM_NODES;
#[no_mangle]
pub static mut nodes: [ObjectNode; LAST_NODE_INDEX] = [ObjectNode::empty(); LAST_NODE_INDEX];

pub unsafe fn kvmem_setup() {
    print!("mm: setting up kernel allocator (nodes={:p}, size={:#x})\n", &nodes, core::mem::size_of_val(&nodes));

    /* setting up initial node */
    nodes[0].addr = 0;
    nodes[0].free = true;
    nodes[0].size = (-1isize) as usize;
    nodes[0].next = LAST_NODE_INDEX;

    /* We have to set qnode to an arbitrary value since
     * enqueue will use kmalloc which would try to enqueue
     * M_QNODE type if qnode == NULL, getting us in an
     * infinite loop
     */
    let m_qnode = &M_QNODE as *const _ as *mut MallocType;
    core::ptr::write_volatile(&mut (*m_qnode).qnode, 0xDEADBEEF as *mut QueueNode<MallocType>);
    (*m_qnode).qnode = (*malloc_types).enqueue(m_qnode);
}

static mut first_free_node: usize = 0;
unsafe fn get_node() -> usize {
    for i in first_free_node..(LAST_NODE_INDEX as usize) {
        if nodes[i].size == 0 {
            return i;
        }
    }

    panic!("Can't find an unused node");
}

unsafe fn release_node(i: usize) {
    if !nodes[i].objtype.is_null() {
        (* nodes[i].objtype).nr -= 1;
    }

    //memset(&nodes[i], 0, sizeof(struct kvmem_node));

    nodes[i].free = true;
}

unsafe fn get_first_fit_free_node(size: usize) -> usize {
    let mut i = first_free_node;

    while !nodes[i].free || (nodes[i].size as usize) < size {
        if nodes[i].next == LAST_NODE_INDEX {
            print!("cannot find a free node\n");
            panic!("Can't find a free node");
        }

        i = nodes[i].next as usize;
    }

    return i;
}

unsafe fn print_node(i: usize) {
    /*
    print!(b"Node[%d]\n\0".as_ptr(), i);
    print!(b"   |_ Addr   : %x\n\0".as_ptr(), node_addr!(nodes[i]));
    print!(b"   |_ free?  : %s\n\0".as_ptr(), if nodes[i].free { b"yes\0".as_ptr() } else { b"no\0".as_ptr() });
    print!(b"   |_ Size   : %d B [ %d KiB ]\n\0".as_ptr(),
        node_size!(nodes[i]), node_size!(nodes[i])/1024);
    print!(b"   |_ Next   : %d\n\0".as_ptr(), nodes[i].next);
    */
}

pub unsafe fn kmalloc(size: usize, objtype: *const MallocType, flags: usize) -> *mut u8 {
    //printk(b"kmalloc(size: %d, type: %p, flags: 0x%x)\n\0".as_ptr(), size, objtype, flags);

    let objtype = objtype as *mut MallocType;

    /* round size to 4-byte units */
    let size = (size + 3)/4;

    /* Look for a first fit free node */
    let i = get_first_fit_free_node(size);

    //printk(b"allocated node %d\n\0".as_ptr(), i);

    /* Mark it as used */
    nodes[i].free = false;

    /* Split the node if necessary */
    if nodes[i].size as usize > size {
        let n = get_node();

        nodes[n].addr = nodes[i].addr + size;
        nodes[n].free = true;
        nodes[n].size = nodes[i].size - size;
        nodes[n].next = nodes[i].next;

        nodes[i].next = n;
        nodes[i].size = size;
    }

    nodes[i].objtype = objtype;
    (*objtype).nr += 1;

    (*objtype).total += node_size!(nodes[i]) as usize;

    kvmem_used += node_size!(nodes[i]) as usize;
    kvmem_obj_cnt += 1;

    let map_base = page_align!(node_addr!(nodes[i]));
    let map_end  = page_round!(node_addr!(nodes[i]) + node_size!(nodes[i]));
    let map_size = (map_end - map_base)/PAGE_SIZE;

    //printk(b"map_size = %d\n\0".as_ptr(), map_size);

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
        (*objtype).qnode = (*malloc_types).enqueue(objtype);
    }

    let obj = node_addr!(nodes[i]);

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
        /* That's not even allocatable */
        return;
    }

    /* Look for the node containing _ptr -- merge sequential free nodes */
    let mut cur_node = 0;
    let mut prev_node = 0;

    while ptr != node_addr!(nodes[cur_node]) {
        /* check if current and previous node are free */
        if cur_node != 0 && nodes[cur_node].free && nodes[prev_node].free {
            /* check for overflow */
            if nodes[cur_node].size + nodes[prev_node].size <= MAX_NODE_SIZE {
                nodes[prev_node].size += nodes[cur_node].size;
                nodes[prev_node].next  = nodes[cur_node].next;
                release_node(cur_node);
                cur_node = nodes[prev_node].next;
                continue;
            }
        }

        prev_node = cur_node;
        cur_node = nodes[cur_node].next;

        if cur_node == LAST_NODE_INDEX {
            /* trying to free unallocated node */
            return;
        }
    }

    if nodes[cur_node].free {
        /* node is already free, dangling pointer? */
        print!("double free detected at {:p}\n", ptr as *const u8);

        if !nodes[cur_node].objtype.is_null() {
            print!("object type: {}\n", cstr((*nodes[cur_node].objtype).name));
        }

        panic!("double free");
    }

    /* first we mark our node as free */
    nodes[cur_node].free = true;

    if !nodes[cur_node].objtype.is_null() {
        (*nodes[cur_node].objtype).total -= node_size!(nodes[cur_node]);
        (*nodes[cur_node].objtype).nr -= 1;

        nodes[cur_node].objtype = core::ptr::null_mut();
    }

    //if (debug_kmalloc) {
    //    printk("NODE_SIZE %d\n", NODE_SIZE(nodes[cur_node]));
    //}

    kvmem_used -= node_size!(nodes[cur_node]);
    kvmem_obj_cnt -= 1;

    /* now we merge all free nodes ahead -- except the last node */
    while nodes[cur_node].next < LAST_NODE_INDEX && nodes[cur_node].free {
        /* check if current and previous node are free */
        if cur_node != 0 && nodes[cur_node].free && nodes[prev_node].free {
            /* check for overflow */
            if nodes[cur_node].size + nodes[prev_node].size <= MAX_NODE_SIZE {
                nodes[prev_node].size += nodes[cur_node].size;
                nodes[prev_node].next  = nodes[cur_node].next;
                release_node(cur_node);
                cur_node = nodes[prev_node].next;
                continue;
            }
        }

        prev_node = cur_node;
        cur_node = nodes[cur_node].next;
    }

    cur_node = 0;
    while nodes[cur_node].next < LAST_NODE_INDEX {
        if nodes[cur_node].free {
            //struct vm_entry vm_entry = {0};

            //vm_entry.paddr = 0;
            //vm_entry.base  = NODE_ADDR(nodes[cur_node]);
            //vm_entry.size  = NODE_SIZE(nodes[cur_node]);
            //vm_entry.flags = VM_KRW;

            let vaddr = node_addr!(nodes[cur_node]);
            let size  = node_size!(nodes[cur_node]);

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

        cur_node = nodes[cur_node].next;
    }
}

/*
void dump_nodes(void)
{
    printk("Nodes dump\n");
    unsigned i = 0;
    while (i < LAST_NODE_INDEX) {
        print_node(i);
        if (nodes[i].next == LAST_NODE_INDEX) break;
        i = nodes[i].next;
    }
}
*/
