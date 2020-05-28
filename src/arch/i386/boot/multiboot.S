.section ".boot.text", "x"

/*
 *  multiboot definitions
 */

#define MULTIBOOT_SEARCH                        8192
#define MULTIBOOT_HEADER_MAGIC                  0x1BADB002
#define MULTIBOOT_BOOTLOADER_MAGIC              0x2BADB002
#define MULTIBOOT_UNSUPPORTED                   0x0000fffc
#define MULTIBOOT_MOD_ALIGN                     0x00001000
#define MULTIBOOT_INFO_ALIGN                    0x00000004
#define MULTIBOOT_PAGE_ALIGN                    0x00000001
#define MULTIBOOT_MEMORY_INFO                   0x00000002
#define MULTIBOOT_VIDEO_MODE                    0x00000004
#define MULTIBOOT_AOUT_KLUDGE                   0x00010000
#define MULTIBOOT_INFO_MEMORY                   0x00000001
#define MULTIBOOT_INFO_BOOTDEV                  0x00000002
#define MULTIBOOT_INFO_CMDLINE                  0x00000004
#define MULTIBOOT_INFO_MODS                     0x00000008
#define MULTIBOOT_INFO_AOUT_SYMS                0x00000010
#define MULTIBOOT_INFO_ELF_SHDR                 0X00000020
#define MULTIBOOT_INFO_MEM_MAP                  0x00000040
#define MULTIBOOT_INFO_DRIVE_INFO               0x00000080
#define MULTIBOOT_INFO_CONFIG_TABLE             0x00000100
#define MULTIBOOT_INFO_BOOT_LOADER_NAME         0x00000200
#define MULTIBOOT_INFO_APM_TABLE                0x00000400
#define MULTIBOOT_INFO_VIDEO_INFO               0x00000800

#if MULTIBOOT_GFX
#define MULTIBOOT_GFX_ENABLE    MULTIBOOT_VIDEO_MODE
#else
#define MULTIBOOT_GFX_ENABLE    0
#endif

#define MULTIBOOT_FLAGS  MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO | MULTIBOOT_GFX_ENABLE
#define MULTIBOOT_CHKSUM -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_FLAGS)

/*
 *  multiboot lookup header
 */

/* struct multiboot_header */
.align 8
.long MULTIBOOT_HEADER_MAGIC    /* magic */
.long MULTIBOOT_FLAGS           /* flags */
.long MULTIBOOT_CHKSUM          /* checksum */
.long 0                         /* header_addr */
.long 0                         /* heload_addr */
.long 0                         /* heload_end_addr */
.long 0                         /* hebss_end_addr */
.long 0                         /* heentry_addr */

.long 0                         /* mode_type */
.long 1024                      /* mowidth */
.long 768                       /* moheight */
.long 32                        /* modepth */

/*
 * entry point of the bootloader
 */

.extern x86_bootstrap
.global _start
_start:
    movl %eax, (multiboot_signature)
    movl %ebx, (multiboot_info)

	/* align stack to 16 byte */
    and  $-16, %esp

    mov  $0, %ebp

    cli

    lea  x86_bootstrap, %eax
	call *%eax
    jmp .

.global multiboot_signature
.global multiboot_info

multiboot_signature: .long 0
multiboot_info: .long 0

/* vim: ft=gas:
*/
