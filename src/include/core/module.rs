use prelude::*;

pub macro module_init {
    ($name:ident, $init:expr, $fini:expr) => {
        #[used]
        #[link_section = ".module.init"]
        static module_init: Option<unsafe fn() -> isize> = $init;

        #[used]
        #[link_section = ".module.fini"]
        static module_fini: Option<unsafe fn() -> isize> = $fini;
    }
}
