use crate::kpanic;
use crate::macros::*;

#[no_mangle]
pub extern "C" fn handle_syscall() {
    kpanic!(b"unimplemented!\n");
}
