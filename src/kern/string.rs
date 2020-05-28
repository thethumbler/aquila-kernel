use prelude::*;

use crate::include::mm::kvmem::*;
use crate::{malloc_declare};

malloc_declare!(M_BUFFER);

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let n = n as isize;
    if (dest as usize) < (src as usize) {
        for i in 0..n {
            *dest.offset(i) = *src.offset(i);
        }
    } else {
        for i in (0..n).rev() {
            *dest.offset(i) = *src.offset(i);
        }
    }

    return dest;
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..(n as isize) {
        *dst.offset(i) = *src.offset(i);
    }

    return dst;
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, chr: u8, n: usize) -> *mut u8 {
    for i in 0..(n as isize) {
        *dst.offset(i) = chr;
    }

    return dst;
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(p1: *const u8, p2: *const u8, n: usize) -> isize {
    for i in 0..(n as isize) {
        if *p1.offset(i) != *p2.offset(i) {
            return *p1.offset(i) as isize - *p2.offset(i) as isize;
        }
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> isize {
    if s1.is_null() || s2.is_null() {
        /* FIXME */
        return 0;
    }

    let mut s1 = s1;
    let mut s2 = s2;

    while *s1 != b'\0' && *s2 != b'\0' && *s1 == *s2 {
        s1 = s1.offset(1);
        s2 = s2.offset(1);
    }

    return *s1 as isize - *s2 as isize;
}

#[no_mangle]
pub unsafe extern "C" fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> isize {
    let mut s1 = s1;
    let mut s2 = s2;
    let mut n = n;

    while n != 0 && *s1 != b'\0' && *s2 != b'\0' && *s1 == *s2 {
        s1 = s1.offset(1);
        s2 = s2.offset(1);
        n -= 1;
    }

    return *s1 as isize - *s2 as isize;
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut ss = s;

    while *ss != b'\0' {
        ss = ss.offset(1);
    }

    return ss as usize - s as usize;
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dst: *mut u8, src: *const u8) -> *mut u8 {
    let retval = dst;

    let mut dst = dst;
    let mut src = src;
    
    while *src != b'\0' {
        *dst = *src;

        src = src.offset(1);
        dst = dst.offset(1);
    }

    *dst = *src;    /* NULL terminator */

    return retval;
}

#[no_mangle]
pub unsafe extern "C" fn strdup(s: *const u8) -> *mut u8 {
    let len = strlen(s);
    let dst = kmalloc(len + 1, &M_BUFFER, 0);

    return memcpy(dst, s, len + 1);
}

pub unsafe fn tokenize(s: *const u8, c: u8) -> *mut *mut u8 {
    if s.is_null() || *s == b'\0' {
        return core::ptr::null_mut();
    }

    let mut s = s;

    while *s == c {
        s = s.offset(1);
    }

    let tokens = strdup(s);

    if tokens.is_null() {
        return core::ptr::null_mut();
    }

    let len = strlen(s);

    if len == 0 {
        let ret = kmalloc(core::mem::size_of::<*mut u8>(), &M_BUFFER, 0) as *mut *mut u8;
        *ret = core::ptr::null_mut();
        return ret;
    }

    let mut count = 0;
    for i in 0..(len as isize) {
        if *tokens.offset(i) == c {
            *tokens.offset(i) = b'\0';
            count += 1;
        }
    }

    if *s.offset(len as isize - 1) != c {
        count += 1;
    }
    
    let ret = kmalloc(core::mem::size_of::<*mut u8>() * (count + 1), &M_BUFFER, 0) as *mut *mut u8;

    let mut j = 0;
    *ret.offset(j) = tokens;
    j += 1;

    for i in 0..(len as isize - 1) {
        if *tokens.offset(i) == b'\0' {
            *ret.offset(j) = tokens.offset(i+1);
            j += 1;
        }
    }

    *ret.offset(j) = core::ptr::null_mut();

    return ret;
}

pub unsafe fn free_tokens(ptr: *mut *mut u8) {
    if ptr.is_null() {
        return;
    }

    if !(*ptr).is_null() {
        kfree(*ptr);
    }

    kfree(ptr as *mut u8);
}
