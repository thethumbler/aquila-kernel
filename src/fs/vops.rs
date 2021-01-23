use prelude::*;
use fs::*;
use dev::*;
use dev::kdev::*;
use mm::*;

pub fn iget(superblock: &mut Node, ino: ino_t) -> Result<&'static mut Node, Error> {
    if superblock.fs.is_none() {
        return Err(Error::ENOSYS);
    }

    unsafe {
        match superblock.fs.as_ref().unwrap().iget {
            Some(f) => f(superblock, ino),
            None => Err(Error::ENOSYS),
        }
    }
}

pub unsafe fn vfs_map(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> isize {
    if vm_entry.is_null() || (*vm_entry).vm_object.is_null() || (*(*vm_entry).vm_object).objtype != VMOBJ_FILE as isize {
        return -EINVAL;
    }

    let mut vnode = (*(*vm_entry).vm_object).p as *mut Node;

    if vnode.is_null() || (*vnode).fs.is_none() {
        return -EINVAL;
    }

    if (*vnode).is_device() {
        return kdev_map(&mut vnode_dev!(vnode), vm_space, vm_entry);
    }

    //if ((*(*vnode).fs).map as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).map(vm_space, vm_entry);
}
