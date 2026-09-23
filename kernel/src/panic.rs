use core::fmt::Write;
use core::panic::PanicInfo;
use crate::uart::Uart;

#[derive(Clone, Copy)]
pub struct TrapContext {
    pub scause: usize,
    pub sepc: usize,
    pub stval: usize,
}

static mut LAST_TRAP: Option<TrapContext> = None;

pub unsafe fn set_trap_context(ctx: TrapContext) {
    LAST_TRAP = Some(ctx);
}

struct RegisterSnapshot {
    ra: usize,
    sp: usize,
    sstatus: usize,
}

impl RegisterSnapshot {
    fn capture() -> Self {
        let ra: usize;
        let sp: usize;
        let sstatus: usize;
        unsafe {
            core::arch::asm!("mv {}, ra", out(reg) ra);
            core::arch::asm!("mv {}, sp", out(reg) sp);
            core::arch::asm!("csrr {}, sstatus", out(reg) sstatus);
        }
        RegisterSnapshot { ra, sp, sstatus }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart = Uart::new();
    let regs = RegisterSnapshot::capture();

    let _ = writeln!(uart, "\n=== KERNEL PANIC ===");
    let _ = writeln!(uart, "{}", info);
    let _ = writeln!(uart, "ra:      {:#018x}", regs.ra);
    let _ = writeln!(uart, "sp:      {:#018x}", regs.sp);
    let _ = writeln!(uart, "sstatus: {:#018x}", regs.sstatus);

    if let Some(ctx) = unsafe { LAST_TRAP } {
        let _ = writeln!(uart, "--- triggering trap ---");
        let _ = writeln!(uart, "scause: {:#x}", ctx.scause);
        let _ = writeln!(uart, "sepc:   {:#x}", ctx.sepc);
        let _ = writeln!(uart, "stval:  {:#x}", ctx.stval);
    }

    crate::qemu_exit::exit_failure(101);
}