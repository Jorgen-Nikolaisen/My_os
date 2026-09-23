use core::arch::global_asm;

global_asm!(
    r#"
    .section .text.boot
    .global _start
    .option norvc

_start:
    la    sp, _stack_top

    la    t0, _bss_start
    la    t1, _bss_end
1:
    bgeu  t0, t1, 2f
    sb    zero, 0(t0)
    addi  t0, t0, 1
    j     1b

2:
    la    t0, mscratch_save
    csrw  mscratch, t0

    la    t0, mtrap_vector
    csrw  mtvec, t0

    # PMP
    li    t0, 0x3fffffffffffff
    csrw  pmpaddr0, t0
    li    t0, 0x0f
    csrw  pmpcfg0, t0

    # medeleg: delegate everything except ecall-from-S (bit 9)
    li    t0, 0xffff
    li    t1, (1 << 9)
    not   t1, t1
    and   t0, t0, t1
    csrw  medeleg, t0

    # mideleg: delegate everything EXCEPT machine timer (bit 7)
    li    t0, 0xffff
    li    t1, (1 << 7)
    not   t1, t1
    and   t0, t0, t1
    csrw  mideleg, t0

    # mie.MTIE
    li    t0, (1 << 7)
    csrw  mie, t0

    # MPP=S, MIE=1
    csrr  t0, mstatus
    li    t1, 0x1800
    not   t1, t1
    and   t0, t0, t1
    li    t1, 0x0800
    or    t0, t0, t1
    li    t1, 0x8
    or    t0, t0, t1
    csrw  mstatus, t0

    la    t0, kernel_main
    csrw  mepc, t0
    mret

    # ============================================================
    # M-mode handler
    # ============================================================
    .balign 4
    .global mtrap_vector
mtrap_vector:
    csrrw sp, mscratch, sp
    sd    t0, 0(sp)
    sd    t1, 8(sp)
    sd    a0, 16(sp)
    sd    a1, 24(sp)

    csrr  t0, mcause
    andi  t1, t0, 0x7FF     # <-- MASK out interrupt bit & reserved bits!

    li    t2, 7             # MTI
    beq   t1, t2, handle_mtimer

    li    t2, 9             # ECALL from S-mode
    beq   t1, t2, handle_ecall

    j     unknown_mtrap

handle_mtimer:
    # Disarm hardware timer first (so it doesn't keep firing)
    li    t0, 0x02004000
    li    t1, -1
    sd    t1, 0(t0)

    # Forward to S-mode via sip.STIP instead of mip
    li    t0, (1 << 5)
    csrs  sip, t0

    j     mtrap_restore

handle_ecall:
    bnez  a0, unknown_mtrap
    li    t0, 0x02004000
    sd    a1, 0(t0)
    csrr  t0, mepc
    addi  t0, t0, 4
    csrw  mepc, t0
    j     mtrap_restore

unknown_mtrap:
    j     unknown_mtrap

mtrap_restore:
    ld    t0, 0(sp)
    ld    t1, 8(sp)
    ld    a0, 16(sp)
    ld    a1, 24(sp)
    csrrw sp, mscratch, sp
    mret

    # ============================================================
    # S-mode vector
    # ============================================================
    .balign 4
    .global trap_vector
trap_vector:
    # Swap sp and sscratch:
    #   from U-mode: sscratch = kernel sp -> sp = kernel stack,
    #                sscratch = user sp (saved into frame below)
    #   from S-mode: sscratch = 0         -> sp = 0, detect & undo
    csrrw sp, sscratch, sp
    beqz  sp, trap_from_kernel

trap_from_user:
    addi  sp, sp, -288
    sd    x5,   40(sp)          # save t0 FIRST — needed as scratch below
    csrr  t0, sscratch          # t0 = user sp
    sd    t0,   16(sp)          # x2 slot = user sp
    li    t0, 0
    csrw  sscratch, t0          # we are "kernel" now, for nested traps
    j     trap_save_regs

trap_from_kernel:
    csrrw sp, sscratch, sp      # undo swap: sp = original kernel sp, sscratch = 0
    addi  sp, sp, -288
    sd    x5,   40(sp)          # save t0 first
    addi  t0, sp, 288
    sd    t0,   16(sp)          # x2 slot = original kernel sp

trap_save_regs:
    sd    x0,    0(sp)
    sd    x1,    8(sp)
    # x2 slot already filled above
    sd    x3,   24(sp)
    sd    x4,   32(sp)
    # x5 slot already filled above
    sd    x6,   48(sp)
    sd    x7,   56(sp)
    sd    x8,   64(sp)
    sd    x9,   72(sp)
    sd    x10,  80(sp)
    sd    x11,  88(sp)
    sd    x12,  96(sp)
    sd    x13, 104(sp)
    sd    x14, 112(sp)
    sd    x15, 120(sp)
    sd    x16, 128(sp)
    sd    x17, 136(sp)
    sd    x18, 144(sp)
    sd    x19, 152(sp)
    sd    x20, 160(sp)
    sd    x21, 168(sp)
    sd    x22, 176(sp)
    sd    x23, 184(sp)
    sd    x24, 192(sp)
    sd    x25, 200(sp)
    sd    x26, 208(sp)
    sd    x27, 216(sp)
    sd    x28, 224(sp)
    sd    x29, 232(sp)
    sd    x30, 240(sp)
    sd    x31, 248(sp)

    csrr  t0, sepc
    sd    t0, 256(sp)
    csrr  t0, sstatus
    sd    t0, 264(sp)
    csrr  t0, scause
    sd    t0, 272(sp)
    csrr  t0, stval
    sd    t0, 280(sp)

    mv    a0, sp
    call  trap_dispatch

        ld    t0, 256(sp)
    csrw  sepc, t0
    ld    t0, 264(sp)
    csrw  sstatus, t0

    # Returning to U-mode? (SPP == 0 in the restored sstatus)
    # Then re-arm sscratch with the kernel stack pointer,
    # so the NEXT trap from user lands on the kernel stack too.
    andi  t0, t0, 0x100
    bnez  t0, trap_restore_regs
    addi  t0, sp, 288
    csrw  sscratch, t0

trap_restore_regs:
    ld    x1,    8(sp)
    ld    x3,   24(sp)
    ld    x4,   32(sp)
    ld    x5,   40(sp)
    ld    x6,   48(sp)
    ld    x7,   56(sp)
    ld    x8,   64(sp)
    ld    x9,   72(sp)
    ld    x10,  80(sp)
    ld    x11,  88(sp)
    ld    x12,  96(sp)
    ld    x13, 104(sp)
    ld    x14, 112(sp)
    ld    x15, 120(sp)
    ld    x16, 128(sp)
    ld    x17, 136(sp)
    ld    x18, 144(sp)
    ld    x19, 152(sp)
    ld    x20, 160(sp)
    ld    x21, 168(sp)
    ld    x22, 176(sp)
    ld    x23, 184(sp)
    ld    x24, 192(sp)
    ld    x25, 200(sp)
    ld    x26, 208(sp)
    ld    x27, 216(sp)
    ld    x28, 224(sp)
    ld    x29, 232(sp)
    ld    x30, 240(sp)
    ld    x31, 248(sp)
    ld    x2,   16(sp)
    sret
    "#
);

extern "C" {
    pub fn trap_vector();
}