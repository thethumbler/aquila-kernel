use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("CRATE_OUT_DIR").unwrap();

    Command::new("ld.lld")
        .arg(&format!("-Tconfigs/i686/link.ld"))
        .arg(&format!("-melf_i386"))
        .arg(&format!("{}/libkernel.a", out_dir))
        .arg(&format!("{}/libkernel.a", out_dir))
        .arg(&format!("--output={}/kernel.elf", out_dir))
        .status()
        .unwrap();

    println!("kernel.elf is ready: {}/kernel.elf", out_dir);
}
