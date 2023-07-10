#![no_std]
#![feature(core_intrinsics)]
#![no_builtins]

pub mod memcpy;
pub mod memset;

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
#[link_section = ".panic_info"]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
