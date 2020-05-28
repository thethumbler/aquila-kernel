use prelude::*;

use kern::string::*;
use crate::kern::print::*;
use crate::{print};

const MAX_ARGS: usize = 128;

#[repr(C)]
#[derive(Copy, Clone)]
struct Kargs {
    key: *const u8,
    value: *const u8,
}

static mut KARGS: [Kargs; MAX_ARGS] = [Kargs {
    key: core::ptr::null(),
    value: core::ptr::null(),
}; MAX_ARGS];

static mut KARGC: usize = 0;

unsafe fn _kargs_parse(cmdline: &[u8]) -> usize {
    print!("kernel: cmdline: {}\n", cstr(cmdline.as_ptr()));

    /* TODO use hashmap */

    if cmdline[0] == b'\0' {
        return 0;
    }

    let tokens = tokenize(cmdline.as_ptr(), b' ');

    let mut token_p = tokens.offset(0);

    while !(*token_p).is_null() {
        let token = *token_p;
        let len = strlen(token);
        let mut parsed = false;

        for i in 0..(len as isize) {
            if *token.offset(i) == b'=' {
                *token.offset(i) = 0;
                KARGS[KARGC].key   = token;
                KARGS[KARGC].value = token.offset(i+1);
                parsed = true;
            }
        }

        if !parsed {
            KARGS[KARGC].key = token;
        }

        KARGC += 1;

        token_p = token_p.offset(1);
    }

    return KARGC;
}

pub unsafe fn kargs_parse(cmdline: *const u8) -> usize {
    if let Some(cmdline) = cmdline.as_ref() {
        let len = strlen(cmdline);
        let cmdline = core::slice::from_raw_parts(cmdline, len+1);
        return _kargs_parse(cmdline);
    }

    return 0;
}

unsafe fn _kargs_get(key: &str) -> Option<&str> {
    for i in 0..KARGC {
        if strcmp(KARGS[i].key, key.as_ptr()) == 0 {
            let len = strlen(KARGS[i].value);
            let slice = core::slice::from_raw_parts(KARGS[i].value, len+1);
            let value = core::str::from_utf8(slice).unwrap();
            return Some(value);
        }
    }

    None
}

pub unsafe fn kargs_get(_key: *const u8, _value: *mut *const u8) -> isize {
    let keylen = strlen(_key);

    let slice = core::slice::from_raw_parts(_key, keylen+1);
    let key = core::str::from_utf8(slice).unwrap();

    let result = _kargs_get(key);

    if let Some(value) = result {
        *_value = value.as_ptr();
        return 0;
    } else {
        return 1;
    }
}
