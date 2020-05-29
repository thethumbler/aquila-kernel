# Aquila Kernel

This is the kernel for AquilaOS re-written in Rust. Currently it's `unsafe` mess with a lot of traces of C code.


# Build

Prerequisites:
* rustc nightly (1.45.0)
* `cargo-post` and `cargo-xbuild` installed (use `$ cargo install cargo-post cargo-xbuild`)
* `lld` (LLVM ld) (use `$ apt install lld`, `$ dnf install lld` or you system package manager)


then run the following command to build `kernel.elf`
```
$ cargo post xbuild --target configs/i686/i686-unknown-none.json --release
```


# Run

The kernel `kernel.elf` is multiboot compliant and you can use any multiboot bootloader to use it.
You need a ramdisk image from aquila in order to use the system, a binary image is provided in this repo
for ease of use (it will be removed later).


```
$ qemu-kvm -kernel target/i686-unknown-none/release/kernel.elf -nographic -initrd initrd.img
```
(on debian based distros, you might need to use `$ qemu-system-i386 -enable-kvm` instead)
