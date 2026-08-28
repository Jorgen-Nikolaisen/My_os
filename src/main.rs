#![no_std]
#![no_main]

use core::arch::global_asm;
use core::fmt::Write;
use core::panic::PanicInfo;

mod uart;

/* Boot assembly entered directly by QEMU */ 
global_asm!(
    r#"
    .section .text.boot 
    .global _start
    .option norvc /*Disable compressed instructions for the boot code */ 
    
_start:
    /*1. Set up stack pointer */ 
    la    sp, _stack_top

    /* 2. Zero the BSS section */ 
    la    t0, _bss_start
    la    t1, _bss_end

1: 
    bgeu  t0, t1, 2f 
    sb    zero, 0(t0)
    addi  t0, t0, 1
    j     1b 

2: 
    li    t0, 0x3fffffffffffff
    csrw  pmpaddr0, t0
    li    t0, 0x0f
    csrw  pmpcfg0, t0

    li    t0, 0xffff 
    csrw  medeleg, t0
    csrw  mideleg, t0 

    csrr  t0, mstatus
    li    t1, 0x1800
    not   t1, t1
    and   t0, t0, t1
    li    t1, 0x0800  
    or    t0, t0, t1
    csrw  mstatus, t0

    la    t0, kernel_main
    csrw  mepc, t0
    
    mret

    "#
);

/* Rust kernel main */ 

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let mut uart = uart::Uart::new();

    writeln!(uart, "Hello from kernel!").unwrap();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut uart = uart::Uart::new();

    // Best effort panic msg
    let _ = writeln!(uart, "Panic: {_info}");
    loop {}
}
