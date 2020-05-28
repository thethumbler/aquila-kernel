fn main() {
    cc::Build::new()
        .file("src/arch/i386/cpu/cpu.S")
        .compile("cpu");

    cc::Build::new()
        .file("src/arch/i386/boot/sys.S")
        .file("src/arch/i386/boot/multiboot.S")
        .compile("boot");

    println!("cargo:rerun-if-changed=src/arch/i386/cpu/cpu.S");
    println!("cargo:rerun-if-changed=src/arch/i386/boot/sys.S");
    println!("cargo:rerun-if-changed=src/arch/i386/boot/multiboot.S");
}
