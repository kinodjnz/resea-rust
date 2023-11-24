#![no_std]
#![feature(core_intrinsics)]
#![no_builtins]

pub mod arch;

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
#[panic_handler]
#[no_mangle]
#[link_section = ".panic_info"]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
