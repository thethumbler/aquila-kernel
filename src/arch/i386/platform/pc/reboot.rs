use prelude::*;

pub unsafe fn platform_reboot() {
    //x86_i8042_reboot();
}

pub unsafe fn arch_reboot() {
    platform_reboot();
} 

