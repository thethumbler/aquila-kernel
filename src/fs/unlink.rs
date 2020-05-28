use prelude::*;
use fs::*;
use mm::*;
use bits::dirent::*;

pub unsafe fn vfs_unlink(path: *mut u8, uio: *mut UserOp) -> isize {
    let mut err = 0;
    let mut p = core::ptr::null_mut();
    let mut tokens = core::ptr::null_mut();

    /* if path is NULL pointer, or path is empty string, return NULL */
    if path.is_null() ||  *path == 0 {
        return -ENOENT;
    }

    let mut _path = core::ptr::null_mut();
    err = vfs_parse_path(path, uio, &mut _path);

    if err != 0 {
        if !_path.is_null() {
            kfree(_path);
        }

        return err;
    }

    /* canonicalize path */
    tokens = tokenize_path(_path);

    /* get mountpoint & path */
    p = vfs_get_mountpoint(tokens);

    let mut dir = (*p).root;
    let mut name = core::ptr::null_mut();
    let mut tok = (*p).tokens;

    while !tok.is_null() {
        let token = *tok;

        if (*tok.offset(1)).is_null() {
            name = token;
            break;
        }

        let mut dirent: DirectoryEntry = core::mem::uninitialized();
        err = vfs_finddir(dir, token, &mut dirent);

        if err != 0 {
            /* FIXME */
            return err;
        }

        err = vfs_vget((*p).root, dirent.d_ino, &mut dir);
        if err != 0 {
            /* FIXME */
            return err;
        }

        tok = tok.offset(1);
    }

    err = vfs_vunlink(dir, name, uio);
    if err != 0 {
        /* FIXME */
        return err;
    }

    free_tokens(tokens);
    kfree(p as *mut u8);
    kfree(_path);

    return 0;
}
