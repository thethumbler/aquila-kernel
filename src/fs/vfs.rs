use prelude::*;

use kern::string::*;
use crate::include::bits::errno::*;
use crate::include::bits::fcntl::*;

use crate::include::fs::vfs::*;
use crate::include::fs::stat::*;

use crate::fs::Vnode;
use crate::include::mm::kvmem::*;

use crate::kern::print::cstr;

use crate::{malloc_define, malloc_declare, print};

malloc_define!(M_VNODE, b"vnode\0", b"vnode structure\0");
malloc_define!(M_VFS_PATH, b"vfs-path\0", b"vfs path structure\0");
malloc_define!(M_VFS_NODE, b"vfs-node\0", b"vfs node structure\0");
malloc_define!(M_FS_LIST, b"fs-list\0", b"filesystems list\0");

malloc_declare!(M_BUFFER);

//static int vfs_log_level = LOG_NONE;
//LOGGER_DEFINE(vfs, vfs_log, vfs_log_level)

/** list of registered filesystems */
pub static mut registered_fs: *mut FilesystemList = core::ptr::null_mut();

/* vfs mountpoints graph */
#[repr(C)]
pub struct VfsNode {
    pub name: &'static str,
    pub children: *mut VfsNode,
    pub next: *mut VfsNode,

    /* reference to node */
    pub vnode: *mut Vnode,
}

pub static mut vfs_graph: VfsNode = VfsNode {
    name: "/",
    children: core::ptr::null_mut(),
    next: core::ptr::null_mut(),

    vnode: core::ptr::null_mut(),
};

/* ================== VFS Graph helpers ================== */

pub static mut vfs_root: *mut Vnode = core::ptr::null_mut();

pub unsafe fn vfs_mount_root(vnode: *mut Vnode) -> isize {
    /* TODO Flush mountpoints */
    vfs_root = vnode;
    vfs_graph.vnode = vnode;
    vfs_graph.children = core::ptr::null_mut();  /* XXX */

    return 0;
}

pub unsafe fn tokenize_path(path: *const u8) -> *mut *mut u8 {
    /* Tokenize slash seperated words in path into tokens */
    tokenize(path, b'/')
}

pub unsafe fn vfs_parse_path(path: *const u8, uio: *mut UserOp, abs_path: *mut *mut u8) -> isize {
    if path.is_null() || *path == 0 {
        return -ENOENT;
    }

    let mut cwd = (*uio).cwd;

    if *path == b'/' {
        /* absolute path */
        cwd = b"/\0".as_ptr() as *mut u8;
    }

    let cwd_len = strlen(cwd) as isize;
    let path_len = strlen(path) as isize;
    let mut buf = kmalloc((cwd_len + path_len + 2) as usize, &M_BUFFER, 0);

    memcpy(buf, cwd, cwd_len as usize);

    *buf.offset(cwd_len) = b'/';
    memcpy(buf.offset(cwd_len + 1), path, path_len as usize);
    *buf.offset(cwd_len + path_len + 1) = 0;

    /* Tokenize slash seperated words in path into tokens */
    let tokens = tokenize(buf, b'/');
    let out = kmalloc((cwd_len + path_len + 1) as usize, &M_BUFFER, 0);

    let mut valid_tokens: [*mut u8; 512] = [core::ptr::null_mut(); 512];
    let mut i = 0;

    let mut token_p = tokens;

    while !(*token_p).is_null() {
        let token = *token_p;

        if *token.offset(0) == b'.' {
            if *token.offset(1) == b'\0' {
                token_p = token_p.offset(1);
                continue;
            }

            if *token.offset(1) == b'.' {
                if *token.offset(2) == b'\0' {
                    if i > 0 {
                        i -= 1;
                        valid_tokens[i] = core::ptr::null_mut();
                    }

                    token_p = token_p.offset(1);
                    continue;
                }
            }

        }

        if *token != 0 {
            valid_tokens[i] = token;
            i += 1;
        }

        token_p = token_p.offset(1);
    }

    valid_tokens[i] = core::ptr::null_mut();

    *out.offset(0) = b'/';

    let mut j = 1;
    let mut token_p = &valid_tokens as *const _ as *mut *mut u8;

    while !(*token_p).is_null() {
        let token = *token_p;
        let len = strlen(token);

        memcpy(out.offset(j), token, len);
        j += len as isize;
        
        *out.offset(j as isize) = b'/';
        j += 1;

        token_p = token_p.offset(1);
    }

    *out.offset(if j > 1 { j -= 1; j } else { 1 }) = 0;

    free_tokens(tokens);
    kfree(buf);

    if !abs_path.is_null() {
        *abs_path = out;
    } else {
        kfree(out);
    }

    return 0;
}

pub unsafe fn vfs_get_mountpoint(tokens: *mut *mut u8) -> *mut VfsPath {
    let mut path = kmalloc(core::mem::size_of::<VfsPath>(), &M_VFS_PATH, 0) as *mut VfsPath;
    (*path).tokens = tokens;

    let mut cur_node = &vfs_graph as *const _ as *mut VfsNode;
    let mut last_target_node = cur_node;

    let mut token_i = 0;
    let mut check_last_node = false;

    let mut token_p = tokens;
    while !(*token_p).is_null() {
        let token = *token_p;

        check_last_node = false;

        if !(*cur_node).vnode.is_null() {
            last_target_node = cur_node;
            (*path).tokens = tokens.offset(token_i);
        }

        if !(*cur_node).children.is_null() {
            cur_node = (*cur_node).children;

            let mut m_node = cur_node; 
            while !m_node.is_null() {
                if cstr(token) == (*m_node).name {
                    cur_node = m_node;
                    check_last_node = true;
                    break;
                }

                m_node = (*m_node).next;
            }

            if !check_last_node {
                /* not found, break! */
                break;
            }
        } else {
            /* no children, break! */
            break;
        }

        token_i += 1;
        token_p = token_p.offset(1);
    }

    if check_last_node && !(*cur_node).vnode.is_null() {
        last_target_node = cur_node;
        (*path).tokens = tokens.offset(token_i);
    }

    (*path).root = (*last_target_node).vnode;

    return path;
}

pub unsafe fn vfs_bind(path: *const u8, target: *mut Vnode) -> isize {
    /* if path is NULL pointer, or path is empty string, or no target return EINVAL */
    if path.is_null() || *path == 0 || target.is_null() {
        return -EINVAL;
    }

    if strcmp(path, b"/\0".as_ptr()) == 0 {
        vfs_mount_root(target);
        return 0;
    }

    /* canonicalize path */
    let tokens = tokenize_path(path);
    
    /* FIXME: should check for existence */

    let mut cur_node = &vfs_graph as *const _ as *mut VfsNode;

    let mut token_p = tokens; 
    while !(*token_p).is_null() {
        let token = *token_p;

        if !(*cur_node).children.is_null() {
            cur_node = (*cur_node).children;

            /* look for token in node children */
            let mut last_node = core::ptr::null_mut() as *mut VfsNode;
            let mut node = cur_node; 
            let mut goto_next = false;

            while !node.is_null() {
                last_node = node;
                if (*node).name == cstr(token) {
                    /* found */
                    cur_node = node;
                    goto_next = true;
                    break;
                }

                node = (*node).next;
            }

            if !goto_next {
                /* not found, create it */
                let mut new_node = kmalloc(core::mem::size_of::<VfsNode>(), &M_VFS_NODE, M_ZERO) as *mut VfsNode;
                if new_node.is_null() {
                    /* TODO */
                }

                (*new_node).name = cstr(strdup(token));
                (*last_node).next = new_node;
                cur_node = new_node;
            }
        } else {
            let mut new_node = kmalloc(core::mem::size_of::<VfsNode>(), &M_VFS_NODE, M_ZERO) as *mut VfsNode;
            if new_node.is_null() {
                /* TODO */
            }

            (*new_node).name = cstr(strdup(token));
            (*cur_node).children = new_node;
            cur_node = new_node;
        }

        token_p = token_p.offset(1);
    }

    (*cur_node).vnode = target;
    return 0;
}

pub unsafe fn vfs_init() {
    //vfs_log(LOG_INFO, "initializing\n");
}

/*
 * \ingroup vfs
 * \brief register a new filesystem handler
 */
pub unsafe fn vfs_install(fs: *mut Filesystem) -> isize {
    let node = kmalloc(core::mem::size_of::<FilesystemList>(), &M_FS_LIST, 0) as *mut FilesystemList;
    if node.is_null() {
        return -ENOMEM;
    }

    (*node).name = (*fs).name;
    (*node).fs   = fs;

    (*node).next = registered_fs;
    registered_fs = node;

    //vfs_log(LOG_INFO, "registered filesystem %s\n", fs->name);
    print!("vfs: registered filesystem {}\n", (*fs).name);

    return 0;
}

/* ================== VFS high level mappings ================== */
pub unsafe fn vfs_perms_check(file: *mut FileDescriptor, uio: *mut UserOp) -> isize {
    if (*uio).uid == 0 {
        /* root */
        return 0;
    }

    let mut read_perms = false;
    let mut write_perms = false;
    let mut exec_perms = false;

    let mode = (*(*file).backend.vnode).mode;
    let uid  = (*(*file).backend.vnode).uid;
    let gid  = (*(*file).backend.vnode).gid;

    /* read permissions */
    read_perms = if (*file).flags & O_ACCMODE == O_RDONLY && (*file).flags & O_ACCMODE != O_WRONLY {
        if      uid == (*uio).uid  { mode & S_IRUSR != 0 }
        else if gid == (*uio).gid  { mode & S_IRGRP != 0 }
        else                       { mode & S_IROTH != 0 }
    } else {
        true
    };

    /* write permissions */
    write_perms = if (*file).flags & (O_WRONLY | O_RDWR) != 0 {
        if      uid == (*uio).uid { mode & S_IWUSR != 0 }
        else if gid == (*uio).gid { mode & S_IWGRP != 0 }
        else                      { mode & S_IWOTH != 0 }
    } else {
        true
    };

    /* execute permissions */
    exec_perms = if (*file).flags & O_EXEC != 0 {
        if      uid == (*uio).uid { mode & S_IXUSR != 0 }
        else if gid == (*uio).gid { mode & S_IXGRP != 0 }
        else                      { mode & S_IXOTH != 0 }
    } else {
        true
    };
    
    if read_perms && write_perms && exec_perms { 0 } else { -EACCES }
}
