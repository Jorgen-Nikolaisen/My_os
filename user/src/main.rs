#![no_std]
#![no_main]

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    let msg = b"Hello from user space\n";
    unsafe {
        syscall_write(1, msg.as_ptr(), msg.len());
    }
    unsafe {
        syscall_exit(0);
    }
}

unsafe fn syscall_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "ecall",
        inlateout("a0") fd => ret,
        in("a1") buf,
        in("a2") len,
        in("a7") 64,    // SYS_WRITE
    );
    ret
}

unsafe fn syscall_exit(code: usize) -> ! {
    core::arch::asm!(
        "ecall",
        in("a0") code,
        in("a7") 93,    // SYS_EXIT
        options(noreturn),
    );

}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop{}
}