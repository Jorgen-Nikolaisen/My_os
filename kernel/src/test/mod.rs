use core::fmt::Write;
use crate::uart::Uart;

pub type TestFn = fn() -> Result<(), &'static str>;

pub struct TestCase {
    pub name: &'static str,
    pub func: TestFn,
}

pub fn run_tests(tests: &[TestCase]) -> ! {
    let mut uart = Uart::new();
    let _ = writeln!(uart, "running {} tests", tests.len());

    let mut passed = 0usize;
    let mut failed = 0usize;

    for t in tests {
        let _ = write!(uart, "test {} ... ", t.name);
        match (t.func)() {
            Ok(()) => {
                let _ = writeln!(uart, "ok");
                passed += 1;
            }
            Err(msg) => {
                let _ = writeln!(uart, "FAILED: {msg}");
                failed += 1;
            }
        }
    }

    let _ = writeln!(uart, "test result: {passed} passed; {failed} failed");

    if failed == 0 {
        crate::qemu_exit::exit_success(0);
    } else {
        crate::qemu_exit::exit_failure(failed as u16);
    }
}

fn test_alloc_is_page_aligned() -> Result<(), &'static str> {
    let addr = unsafe { crate::mm::pmm::frame_allocator().alloc_frame() }.ok_or("out of memory")?;
    if addr % crate::mm::pmm::PAGE_SIZE != 0 {
        return Err("frame not page-aligned");
    }
    Ok(())
}

fn test_free_then_alloc_reuses_frame() -> Result<(), &'static str> {
    let a = unsafe { crate::mm::pmm::frame_allocator().alloc_frame() }.ok_or("out of memory")?;
    unsafe { crate::mm::pmm::frame_allocator().free_frame(a) };
    let b = unsafe { crate::mm::pmm::frame_allocator().alloc_frame() }.ok_or("out of memory")?;
    if a != b {
        return Err("expected freed frame to be reused (free list is LIFO)");
    }
    Ok(())
}

pub static TESTS: &[TestCase] = &[
    TestCase { name: "alloc_is_page_aligned", func: test_alloc_is_page_aligned },
    TestCase { name: "free_then_alloc_reuses_frame", func: test_free_then_alloc_reuses_frame },
];
