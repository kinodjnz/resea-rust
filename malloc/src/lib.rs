#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]
#![feature(asm_const)]
#![feature(generators, generator_trait)]
#![feature(core_intrinsics)]
#![feature(cell_update)]
// #![feature(offset_of)]

#[macro_use]
extern crate klib;

#[macro_use]
extern crate syscall;

pub mod malloc;

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
