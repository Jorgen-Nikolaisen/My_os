use core::arch::global_asm;
use crate::task::context::Context;

global_asm!(
    r#"
    .section .text
    .global context_switch
context_switch:
    # a0 = &mut Context (old)
    # a1 = &Context (new)

    sd   ra,  0(a0)
    sd   sp,  8(a0)
    sd   s0,  16(a0)
    sd   s1,  24(a0)
    sd   s2,  32(a0)
    sd   s3,  40(a0)
    sd   s4,  48(a0)
    sd   s5,  56(a0)
    sd   s6,  64(a0)
    sd   s7,  72(a0)
    sd   s8,  80(a0)
    sd   s9,  88(a0)
    sd   s10, 96(a0)
    sd   s11, 104(a0)

    ld   ra,  0(a1)
    ld   sp,  8(a1)
    ld   s0,  16(a1)
    ld   s1,  24(a1)
    ld   s2,  32(a1)
    ld   s3,  40(a1)
    ld   s4,  48(a1)
    ld   s5,  56(a1)
    ld   s6,  64(a1)
    ld   s7,  72(a1)
    ld   s8,  80(a1)
    ld   s9,  88(a1)
    ld   s10, 96(a1)
    ld   s11, 104(a1)

    ret
    "#
);

global_asm!(
    r#"
    .section .text
    .global switch_and_never_return
switch_and_never_return:
    # a0 = &Context (new). Restore it and jump into its suspended chain.
    ld   ra,  0(a0)
    ld   sp,  8(a0)
    ld   s0,  16(a0)
    ld   s1,  24(a0)
    ld   s2,  32(a0)
    ld   s3,  40(a0)
    ld   s4,  48(a0)
    ld   s5,  56(a0)
    ld   s6,  64(a0)
    ld   s7,  72(a0)
    ld   s8,  80(a0)
    ld   s9,  88(a0)
    ld   s10, 96(a0)
    ld   s11, 104(a0)
    ret        # now executing on the NEXT process's suspended chain
    "#
);


extern "C" {
    pub fn switch_and_never_return(new: *const Context);
    pub fn context_switch(old: *mut Context, new: *const Context);

}
