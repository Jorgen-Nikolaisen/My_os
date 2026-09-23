pub mod context;
pub mod switch;

use crate::mm::pmm;
use self::context::Context;
use self::switch::context_switch;

const STACK_PAGES: usize = 2; // 8 KiB stacks
static mut THREADS: [Thread; 4] = [Thread::empty(); 4];
static mut CURRENT: usize = 0;
static mut THREAD_COUNT: usize = 1;

#[derive(Clone, Copy)]
pub struct Thread {
    pub context: Context,
    pub stack_base: usize,
}

impl Thread {
    pub const fn empty() -> Self {
        Thread {
            context: Context::zero(),
            stack_base: 0,
        }
    }
}

/// Create a new kernel thread that will call `entry()`.
/// # Safety
/// PMM must be initialized. Do not call from an interrupt handler.
pub unsafe fn spawn(entry: fn() -> !) -> Result<(), ()> {
    let idx = THREAD_COUNT;
    if idx >= THREADS.len() {
        return Err(());
    }

    let stack_phys = pmm::frame_allocator()
        .alloc_frames(STACK_PAGES)
        .ok_or(())?;
    let stack_top = stack_phys + STACK_PAGES * pmm::PAGE_SIZE;

    THREADS[idx].stack_base = stack_phys;
    THREADS[idx].context = Context::zero();
    THREADS[idx].context.ra = entry as usize;
    THREADS[idx].context.sp = stack_top;

    THREAD_COUNT += 1;
    Ok(())
}

/// Cooperative yield. Switches to the next runnable thread round-robin.
pub fn yield_now() {
    unsafe {
        let next = (CURRENT + 1) % THREAD_COUNT;
        if next == CURRENT {
            return; // only one thread
        }
        let old = &mut THREADS[CURRENT].context as *mut Context;
        let new = &THREADS[next].context as *const Context;
        CURRENT = next;
        context_switch(old, new);
    }
}
