OUTPUT_FORMAT(elf32-i386)
ENTRY(_start)
/* INPUT(libboot.a libcpu.a libkernel.a) */
OUTPUT(kernel.elf)

SECTIONS {
    LMA = 0x100000;
    . = LMA;

    kernel_start = .;

    .boot.text : ALIGN(0x1000) {
        *(.boot.text)
        *(.boot.rodata)
    }

    .boot.data : ALIGN(0x1000) {
        *(.boot.data)
    }

    .boot.bss (NOLOAD) : ALIGN(0x1000) {
        *(.boot.*)
    }

    boot_end = .;

    _VMA = 0xC0000000;
    . += _VMA;

    .text : AT(ADDR(.text) - _VMA) ALIGN(0x1000) {
        *(.text*)
        *(.rodata*)
    }

    .eh_frame : AT(ADDR(.eh_frame) - _VMA) ALIGN(0x1000) {
        *(.eh_frame*)
    }

    .module.init : AT(ADDR(.module.init) - _VMA) ALIGN(0x1000) {
        module_init = .;
        *(.module.init)
        module_init_end = .;
    }
    
    .data : AT(ADDR(.data) - _VMA) ALIGN(0x1000) {
        *(.data*)
    }
    
    .bss : AT(ADDR(.bss) - _VMA) ALIGN(0x1000) {
        *(.bss*)
    }

    /*
    .rel.plt : AT(ADDR(.rel.plt) - _VMA) ALIGN(0x1000) {
        *(.rel.plt)
    }

    .strtab : AT(ADDR(.strtab) - _VMA) ALIGN(0x1000) {
        *(.strtab)
    }

    .shstrtab : AT(ADDR(.shstrtab) - _VMA) ALIGN(0x1000) {
        *(.shstrtab)
    }
    */
    
    /*
    /DISCARD/ : {
        *(.*)
    }
    */
    
    kernel_end = . - _VMA;
}
