use crate::macros::*;

#[no_mangle]
pub extern "C" fn cramp32_handle_exception() {
    kpanic!(b"Unknown exception\n");
}

#[no_mangle]
pub extern "C" fn cramp32_handle_interrupt() {
    kpanic!(b"unimplemented!");
}
