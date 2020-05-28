use prelude::*;

use crate::include::core::types::*;
use crate::include::core::string::*;
use crate::include::bits::errno::*;
use crate::include::fs::vfs::*;
use dev::dev::*;
use dev::kdev::*;
use crate::include::fs::stat::*;

use crate::mm::*;
use crate::include::mm::mm::*;
use crate::include::mm::vm::*;

use crate::fs::vnode::*;

use crate::{ISDEV, VNODE_DEV, DEV_MAJOR, DEV_MINOR};
use crate::{S_ISCHR, S_ISBLK, S_ISDIR};

pub unsafe fn vfs_vmknod(dir: *mut Vnode, name: *const u8, mode: mode_t, dev: dev_t, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    /* invalid request */
    if dir.is_null() || (*dir).fs.is_null() {
        return -EINVAL;
    }

    /* not a directory */
    if !S_ISDIR!((*dir).mode) {
        return -ENOTDIR;
    }

    /* operation not supported */
    //if ((*(*dir).fs).vops.vmknod as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    let ret = (*dir).vmknod(name, mode, dev, uio, vnode_ref);

    if ret == 0 && !vnode_ref.is_null() && !(*vnode_ref).is_null() {
        (**vnode_ref).refcnt += 1;
    }

    return ret;
}

pub unsafe fn vfs_vcreat(dir: *mut Vnode, name: *const u8, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    return vfs_vmknod(dir, name, S_IFREG, 0, uio, vnode_ref);
}

pub unsafe fn vfs_vmkdir(dir: *mut Vnode, name: *const u8, uio: *mut UserOp, vnode_ref: *mut *mut Vnode) -> isize {
    return vfs_vmknod(dir, name, S_IFDIR, 0, uio, vnode_ref);
}

pub unsafe fn vfs_vunlink(dir: *mut Vnode, name: *const u8, uio: *mut UserOp) -> isize {
    /* invalid request */
    if dir.is_null() || (*dir).fs.is_null() {
        return -EINVAL;
    }

    /* operation not supported */
    //if ((*(*dir).fs).vops.vunlink as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*dir).vunlink(name, uio);
}

pub unsafe fn vfs_vget(super_node: *mut Vnode, ino: ino_t, vnode_ref: *mut *mut Vnode) -> isize {
    let mut err = 0;
    let mut vnode: *mut Vnode = core::ptr::null_mut();

    if super_node.is_null() || (*super_node).fs.is_null() {
        return -EINVAL;
    }

    //if ((*(*super_node).fs).vops.vget as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    err = (*super_node).vget(ino, &mut vnode);

    if err != 0 {
        return err;
    }

    if !vnode.is_null() {
        (*vnode).refcnt += 1;
    }

    if !vnode_ref.is_null() {
        *vnode_ref = vnode;
    }

    return err;
}

pub unsafe fn vfs_map(vm_space: *mut VmSpace, vm_entry: *mut VmEntry) -> isize {
    if vm_entry.is_null() || (*vm_entry).vm_object.is_null() || (*(*vm_entry).vm_object).objtype != VMOBJ_FILE as isize {
        return -EINVAL;
    }

    let mut vnode = (*(*vm_entry).vm_object).p as *mut Vnode;

    if vnode.is_null() || (*vnode).fs.is_null() {
        return -EINVAL;
    }

    if ISDEV!(vnode) {
        return kdev_map(&mut VNODE_DEV!(vnode), vm_space, vm_entry);
    }

    //if ((*(*vnode).fs).vops.map as *const u8).is_null() {
    //    return -ENOSYS;
    //}

    return (*vnode).map(vm_space, vm_entry);
}
