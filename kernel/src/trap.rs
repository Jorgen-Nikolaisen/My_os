use crate::log_info;

#[repr(C)]
pub struct TrapFrame {
    pub regs: [usize; 32],
    pub sepc: usize,
    pub sstatus: usize,
    pub scause: usize,
    pub stval: usize,
}

const EXC_BREAKPOINT: usize = 3;
const EXC_ECALL_U: usize = 8;
const EXC_ECALL_S: usize = 9;
const EXC_INST_PAGE_FAULT: usize = 12;
const EXC_LOAD_PAGE_FAULT: usize = 13;
const EXC_STORE_PAGE_FAULT: usize = 15;

const INT_S_SOFT: usize = 1;
const INT_S_TIMER: usize = 5;
const INT_S_EXTERNAL: usize = 9;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;

#[no_mangle]
extern "C" fn trap_dispatch(tf: &mut TrapFrame) {
    // SUM=1 lets S-mode read user (PTE_U) pages — needed by sys_write.
    unsafe {
        let mut sstatus: usize;
        core::arch::asm!("csrr {}, sstatus", out(reg) sstatus);
        sstatus |= 1 << 18; // SUM
        core::arch::asm!("csrw sstatus, {}", in(reg) sstatus);
    }

    let is_interrupt = (tf.scause as isize) < 0;
    let code = tf.scause & 0xfff;

    if is_interrupt {
        handle_interrupt(tf, code);
    } else {
        handle_exception(tf, code);
    }
}

fn handle_interrupt(_tf: &mut TrapFrame, code: usize) {
    match code {
        INT_S_TIMER => log_info!("trap", "timer tick (sepc={:#x})", _tf.sepc),
        INT_S_SOFT => log_info!("trap", "software interrupt"),
        INT_S_EXTERNAL => log_info!("trap", "external interrupt (PLIC)"),
        _ => panic!("unhandled supervisor interrupt #{code} at sepc={:#x}", _tf.sepc),
    }
}

fn handle_exception(tf: &mut TrapFrame, code: usize) {
    match code {
        EXC_ECALL_U => handle_syscall(tf),
        EXC_ECALL_S => {
            log_info!("trap", "ecall from S-mode");
            tf.sepc += 4;
        }
        EXC_BREAKPOINT => {
            log_info!("trap", "breakpoint at sepc={:#x}", tf.sepc);
            tf.sepc += 2;
        }
        EXC_INST_PAGE_FAULT | EXC_LOAD_PAGE_FAULT | EXC_STORE_PAGE_FAULT => {
            panic!(
                "page fault: code={code} sepc={:#x} stval={:#x}",
                tf.sepc, tf.stval
            );
        }
        _ => panic!(
            "unhandled exception #{code}: sepc={:#x} stval={:#x}",
            tf.sepc, tf.stval
        ),
    }
}

/// a0=x10, a1=x11, a2=x12, a7=x17
fn handle_syscall(tf: &mut TrapFrame) {
    log_info!("syscall", "nr={} a0={:#x} a1={:#x} a2={:#x} sepc={:#x}",
    tf.regs[17], tf.regs[10], tf.regs[11], tf.regs[12], tf.sepc);
    
    let nr = tf.regs[17];
    match nr {
        SYS_WRITE => {
            let ret = sys_write(tf.regs[10], tf.regs[11] as *const u8, tf.regs[12]);
            tf.regs[10] = ret as usize; // return value in a0
            tf.sepc += 4;               // skip the ecall
        }
        SYS_EXIT => {
            log_info!("syscall", "exit({})", tf.regs[10]);
            crate::qemu_exit::exit_success(tf.regs[10] as u16);
        }
        _ => {
            log_info!("syscall", "unknown syscall #{nr}");
            tf.regs[10] = usize::MAX; // -1
            tf.sepc += 4;
        }
    }
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    if fd != 1 {
        return -9; // EBADF
    }
    // NOTE: a real kernel must validate buf/len before touching them.
    // We trust the user for now — SUM=1 makes the read legal.
    let slice = unsafe { core::slice::from_raw_parts(buf, len) };
    crate::uart::Uart::new().write_bytes(slice);
    len as isize
}
