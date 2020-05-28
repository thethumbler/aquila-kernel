use prelude::*;

use crate::{print};

extern "C" {
    static module_init: u8;
    static module_init_end: u8;
}

pub unsafe fn modules_init() -> isize {
    print!("kernel: loading builtin modules\n");

    /* initalize built-in modules */
    let nr = (&module_init_end as *const u8 as usize - &module_init as *const u8 as usize) / core::mem::size_of::<usize>();
    let init = &module_init as *const u8 as *const Option<unsafe fn()>;

    print!("kernel: found {} modules\n", nr);

    for i in 0..nr {
        let func = &*init.offset(i as isize);
        if func.is_some() {
            func.unwrap()();
        }
    }

    return 0;
}

