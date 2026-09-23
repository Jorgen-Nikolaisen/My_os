OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    /* Must match USER_BASE in kernel/src/main.rs */
    . = 0x80400000;

    .text : {
        KEEP(*(.text.entry))    /* _start first, immune to --gc-sections */
        *(.text .text.*)
    }

    .rodata : ALIGN(4) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    .data : ALIGN(4) {
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.sdata .sdata.*)
        *(.data .data.*)
    }

    .bss : ALIGN(4) {
        *(.sbss .sbss.*)
        *(.bss .bss.*)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
        *(.debug_*)
    }
}
