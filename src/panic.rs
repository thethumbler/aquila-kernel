use prelude::*;

use crate::prelude::*;

#[lang = "panic_impl"]
pub unsafe extern fn rust_begin_panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        print!("panic occurred in file '{}' at line {}:\n", location.file(),
            location.line());
    } else {
        print!("panic occurred but can't get location information...:\n");
    }

    core::fmt::write(&mut crate::kern::print::RCONSOLE, *info.message().unwrap()).unwrap();
   
    loop {}
}
