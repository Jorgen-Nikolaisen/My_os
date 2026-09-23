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
#[link_section = ".user"]
static USER_PROGRAM: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/user.bin.o")).len()] =
    *include_bytes!(concat!(env!("OUT_DIR"), "/user.bin.o"));

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

extern "C" {
    static _kernel_end: u8;
    static _user_start: u8;
    static _user_end: u8;
}

const RAM_START: usize = 0x8000_0000;
const RAM_SIZE: usize = 128 * 1024 * 1024; // QEMU's default; pass -m to change
const RAM_END: usize = RAM_START + RAM_SIZE;
const USER_BASE: usize = 0x8040_0000; // must match kernel/linker.ld + user/linker.ld

fn thread_a() -> ! {
    loop {
        log_info!("task", "Thread A running");
    }
}

fn thread_b() -> ! {
    loop {
        log_info!("task", "Thread B running");
    }
}

/// Ask our mini-SBI to set the next timer.
unsafe fn set_timer(next: u64) {
    core::arch::asm!(
        "ecall",
        in("a0") 0u64,
        in("a1") next,
    );
}

/// Read 64-bit mtime from CLINT
unsafe fn clint_mtime() -> u64 {
    core::ptr::read_volatile(0x0200_BFF8 as *const u64)
}



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

    let user_start = unsafe { &_user_start as *const _ as usize };
    let user_size = unsafe { &_user_end as *const _ as usize - user_start };

    log_info!("boot", "user binary at {:#x}, size {}", user_start, user_size);

    // Allocate a user stack
    let user_stack_phys = unsafe { mm::pmm::frame_allocator().alloc_frames(2).expect("oom") };
    let user_stack_top = user_stack_phys + 2 * mm::pmm::PAGE_SIZE;

    unsafe {
        mm::vmm::identity_map_range(user_start, user_start + user_size, mm::vmm::PTE_R | mm::vmm::PTE_X | mm::vmm::PTE_U);
        mm::vmm::identity_map_range(user_stack_phys, user_stack_top, mm::vmm::PTE_R | mm::vmm::PTE_W | mm::vmm::PTE_U);
    }

    unsafe { _enter_usermode(user_start, user_stack_top) };

    // NEVER REACHED — but satisfy compiler
    loop {}
}