pub mod context;
pub mod switch;

use crate::log_info;
use crate::mm::pmm;
use context::Context;
use switch::context_switch;

const MAX_PROCS: usize = 8;
const USER_STACK_PAGES: usize = 2;
const KSTACK_PAGES: usize = 8; // 32 KiB kernel stack per process

// slot 0 = kernel_main (idle). User processes are slots 1..
static mut PROCS: [Proc; MAX_PROCS] = [Proc::EMPTY; MAX_PROCS];
static mut CURRENT: usize = 0;
static mut PROC_COUNT: usize = 1;

extern "C" {
    fn _enter_usermode(user_pc: usize, user_sp: usize) -> !;  // from main.rs global_asm
}

#[derive(Clone, Copy, PartialEq)]
pub enum State { Idle, Ready, Dead }

#[derive(Clone, Copy)]
pub struct Proc {
    pub pid: usize,
    pub state: State,
    pub ctx: Context,              // kernel call-chain context
    pub user_entry: usize,
    pub user_sp: usize,
    pub user_img: (usize, usize),   // [lo, hi) — for EFAULT checks
    pub user_stack: (usize, usize),
}

impl Proc {
    const EMPTY: Proc = Proc {
        pid: 0, state: State::Idle, ctx: Context::zero(),
        user_entry: 0, user_sp: 0, user_img: (0, 0), user_stack: (0, 0),
    };
}

/// Create a user process from an embedded image.
pub unsafe fn spawn_user(pid: usize, img_lo: usize, img_hi: usize) -> Result<(), ()> {
    let count = core::ptr::read(core::ptr::addr_of!(PROC_COUNT));
    if count >= MAX_PROCS { return Err(()); }

    // per-process user stack (mapped U)
    let stack_lo = pmm::frame_allocator().alloc_frames(USER_STACK_PAGES).ok_or(())?;
    let stack_hi = stack_lo + USER_STACK_PAGES * pmm::PAGE_SIZE;
    crate::mm::vmm::identity_map_range(
        stack_lo, stack_hi,
        crate::mm::vmm::PTE_R | crate::mm::vmm::PTE_W | crate::mm::vmm::PTE_U);

    // per-process kernel stack (already identity-mapped by vmm init)
    let kstack = pmm::frame_allocator().alloc_frames(KSTACK_PAGES).ok_or(())?;
    let kstack_top = kstack + KSTACK_PAGES * pmm::PAGE_SIZE;

    (*core::ptr::addr_of_mut!(PROCS))[count] = Proc {
        pid, state: State::Ready,
        ctx: Context { ra: proc_first_run as usize, sp: kstack_top, ..Context::zero() },
        user_entry: img_lo, user_sp: stack_hi,
        user_img: (img_lo, img_hi), user_stack: (stack_lo, stack_hi),
    };
    core::ptr::write(core::ptr::addr_of_mut!(PROC_COUNT), count + 1);
    Ok(())
}

/// Entry point of a freshly-switched-to process: drop into user mode.
/// Never returns; arms sscratch with THIS process's kernel sp.
extern "C" fn proc_first_run() -> ! {
    unsafe {
        let cur = core::ptr::read(core::ptr::addr_of!(CURRENT));
        let p = &(*core::ptr::addr_of!(PROCS))[cur];
        let (pc, sp) = (p.user_entry, p.user_sp);
        _enter_usermode(pc, sp);
    }
}

/// Round-robin yield. Called from SYS_yield (inside a trap, frame already saved
/// on the current process's kernel stack) or once from kernel_main to start.
pub fn yield_now() {
    unsafe {
        let cur = core::ptr::read(core::ptr::addr_of!(CURRENT));
        let Some(next) = next_runnable(cur) else { return; };
        if cur != 0 {
            (*core::ptr::addr_of_mut!(PROCS))[cur].state = State::Ready;
        }
        core::ptr::write(core::ptr::addr_of_mut!(CURRENT), next);
        // THE switch: we freeze mid-call on OUR kstack; we resume here only
        // when some other process switches back to us.
        context_switch(&mut (*core::ptr::addr_of_mut!(PROCS))[cur].ctx as *mut Context,
                       &(*core::ptr::addr_of!(PROCS))[next].ctx as *const Context);
    }
}

pub fn exit_current(code: usize) -> ! {
    log_info!("task", "process exited with code {code}");
    unsafe {
        let cur = core::ptr::read(core::ptr::addr_of!(CURRENT));
        (*core::ptr::addr_of_mut!(PROCS))[cur].state = State::Dead;
        match next_runnable(cur) {
            Some(next) => {
                core::ptr::write(core::ptr::addr_of_mut!(CURRENT), next);
                switch::switch_and_never_return(
                    &(*core::ptr::addr_of!(PROCS))[next].ctx as *const Context);
            }
            None => {
                log_info!("task", "no processes left, shutting down");
                crate::qemu_exit::exit_success(0);
            }
        }
    }
    unreachable!()
}

/// Memory ranges of the current process, for syscall pointer validation.
pub fn current_mm() -> ((usize, usize), (usize, usize)) {
    unsafe {
        let cur = core::ptr::read(core::ptr::addr_of!(CURRENT));
        let p = &(*core::ptr::addr_of!(PROCS))[cur];
        (p.user_img, p.user_stack)
    }
}

unsafe fn next_runnable(from: usize) -> Option<usize> {
    let count = core::ptr::read(core::ptr::addr_of!(PROC_COUNT));
    for off in 1..count {
        let idx = (from + off) % count;
        if idx != 0 && (*core::ptr::addr_of!(PROCS))[idx].state == State::Ready {
            return Some(idx);
        }
    }
    None
}