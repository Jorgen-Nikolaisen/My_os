#![no_std]
use core::arch::asm;

pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
    let ret: isize;
    unsafe { asm!("ecall", inlateout("a0") fd => ret, in("a1") buf, in("a2") len, in("a7") 64u64) };
    ret
}

pub fn println(s: &str) { write(1, s.as_ptr(), s.len()); }

pub fn yield_() { unsafe { asm!("ecall", in("a7") 124u64) }; }

pub fn exit(code: usize) -> ! {
    unsafe { asm!("ecall", in("a0") code, in("a7") 93u64, options(noreturn)) };
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { exit(101) }
