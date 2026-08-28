
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
}
