
OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80000000;

SECTIONS
{
	. = BASE_ADDRESS;

	.text : ALIGN(4096) {
		*(.text.boot)
		*(.text .text.*)
	}

	.rodata : ALIGN(4096) {
		*(.rodata .rodata.*)
		*(.srodata .srodata.*)
	}

	/* Initialized data */ 
	.data : ALIGN(4096) {
		*(.data .data.*)
		*(.sdata .sdata.*)
	}

	/* Uninitialized data / zero init */ 
	.bss : ALIGN(4096) {
		_bss_start = .;
		*(.bss .bss.*)
		*(.sbss .sbss.*)
		*(COMMON)
		_bss_end = .;
	}

	/DISCARD/ : {
		*(.eh_frame)
		*(.comment)
	}

	_stack_bottom = .;
	. = . + 64K;
	_stack_top = .;

	_kernel_end = .;


    /* User images at FIXED addresses — must match build.rs USER_BINS */
    .user_a 0x80400000 : {
        _user_a_start = .;
        KEEP(*(.user_a))
        _user_a_end = .;
    }
    .user_b 0x80402000 : {
        _user_b_start = .;
        KEEP(*(.user_b))
        _user_b_end = .;
    }
}
