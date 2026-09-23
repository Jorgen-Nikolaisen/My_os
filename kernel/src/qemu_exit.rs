const VIRT_TEST: usize = 0x10_0000;

const FINISHER_FAIL: u32 = 0x3333;
const FINISHER_PASS: u32 = 0x5555;

fn write_finisher(status: u32, code: u16) -> ! {
    let val = ((code as u32) << 16) | status;
    unsafe { core::ptr::write_volatile(VIRT_TEST as *mut u32, val) };
    loop {} // the shutdown request isn't instantaneous
}

pub fn exit_success(code: u16) -> ! {
    write_finisher(FINISHER_PASS, code)
}

pub fn exit_failure(code: u16) -> ! {
    write_finisher(FINISHER_FAIL, code)
}
