#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]
#![feature(asm_const)]

#[macro_use]
extern crate klib;

pub mod init;
mod syscall;

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
#[link_section = ".panic_info"]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
