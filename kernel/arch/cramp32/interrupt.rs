use super::timer;
use crate::macros::*;
use crate::task;

#[no_mangle]
pub extern "C" fn cramp32_handle_exception() {
    kpanic!(b"Unexpected exception\n");
}

#[no_mangle]
pub extern "C" fn cramp32_handle_interrupt(_a0: u32, _a1: u32, mcause: u32) {
    match mcause {
        0x80000007 => {
            timer::reload();
            task::handle_timer_irq();
        }
        _ => {
            kpanic!(b"unimplemented!\n");
        }
    }
}
