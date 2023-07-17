#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]
#![feature(asm_const)]
#![feature(generators, generator_trait)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_uninit_array)]
#![feature(sync_unsafe_cell)]

extern crate alloc;

#[macro_use]
extern crate klib;

#[macro_use]
extern crate syscall;

mod generator;
pub mod init;

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
