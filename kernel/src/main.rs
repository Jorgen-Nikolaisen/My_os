#![no_std]
#![no_main]
use core::arch::global_asm;


mod boot;
mod log;
mod mm;
mod panic;
mod qemu_exit;
mod test;
mod trap;
mod uart;
mod task;

#[used]
#[link_section = ".user_a"]
static HELLO_A: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/hello_a.bin")).len()] =
    *include_bytes!(concat!(env!("OUT_DIR"), "/hello_a.bin"));

#[used]
#[link_section = ".user_b"]
static HELLO_B: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/hello_b.bin")).len()] =
    *include_bytes!(concat!(env!("OUT_DIR"), "/hello_b.bin"));

extern "C" {
    static _kernel_end: u8;
    static _user_a_start: u8;
    static _user_a_end: u8;
    static _user_b_start: u8;
    static _user_b_end: u8;
}

const USER_BASE: usize = 0x8040_0000; // images live above this; PMM stays below

#[no_mangle]
static mut mscratch_save: [u64; 8] = [0; 8];

extern "C" {
    fn _enter_usermode(user_pc: usize, user_sp: usize) -> !;
}

global_asm!(
    r#"
    .section .text
    .global _enter_usermode
    .balign 4
_enter_usermode:
    # a0 = user_pc, a1 = user_sp
    csrw sepc, a0
    csrw sstatus, zero        # SPP=0 -> sret enters U-mode; SIE=0; SUM=0
    csrw sscratch, sp         # NEW: remember the kernel sp for the trap vector
    mv   sp, a1               # switch to the user stack
    sret
    "#
);



#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        core::arch::asm!("csrw stvec, {}", in(reg) boot::trap_vector as *const () as usize);
    }

    let kernel_end = unsafe { &_kernel_end as *const u8 as usize };
    unsafe { mm::pmm::frame_allocator().init(kernel_end, USER_BASE); } 

    log_info!("boot", "niko-os starting");
    unsafe { mm::vmm::init() };
    log_info!("vmm", "MMU enabled (Sv39)");

    log_info!("boot", "preparing to enter user mode");

    let a_lo = unsafe { &_user_a_start as *const u8 as usize };
    let a_hi = unsafe { &_user_a_end as *const u8 as usize };
    let b_lo = unsafe { &_user_b_start as *const u8 as usize };
    let b_hi = unsafe { &_user_b_end as *const u8 as usize };

    unsafe {
        mm::vmm::identity_map_range(a_lo, a_hi, mm::vmm::PTE_R | mm::vmm::PTE_X | mm::vmm::PTE_U);
        mm::vmm::identity_map_range(b_lo, b_hi, mm::vmm::PTE_R | mm::vmm::PTE_X | mm::vmm::PTE_U);
        task::spawn_user(1, a_lo, a_hi).unwrap();
        task::spawn_user(2, b_lo, b_hi).unwrap();
    }
    log_info!("task", "starting 2 processes");
    task::yield_now();   // switch into proc 1; returns only if nothing runnable
    loop {}
    
}