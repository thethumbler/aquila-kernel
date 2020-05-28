use prelude::*;
use fs::*;

pub unsafe fn posix_file_can_read(file: *mut FileDescriptor, size: size_t) -> isize {
    ((*file).offset as usize + size < (*(*file).backend.vnode).size) as isize
}

pub unsafe fn posix_file_can_write(file: *mut FileDescriptor, size: size_t) -> isize {
    ((*file).offset as usize + size < (*(*file).backend.vnode).size) as isize
}

pub unsafe fn posix_file_eof(file: *mut FileDescriptor) -> isize {
    ((*file).offset as usize >= (*(*file).backend.vnode).size) as isize
}
