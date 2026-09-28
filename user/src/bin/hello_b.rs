#![no_std]
#![no_main]

use user_lib::{println, yield_, exit};

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    for _ in 0..3 {
        println("hello B\n");
        yield_();
    }
    exit(0);
}