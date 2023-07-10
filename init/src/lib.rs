#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]
#![feature(asm_const)]
#![feature(generators, generator_trait)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate klib;

mod generator;
pub mod init;
mod syscall;

use core::panic::PanicInfo;
#[panic_handler]
// #[no_mangle]
// #[link_section = ".panic_info"]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
