#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]
#![feature(asm_const)]
#![feature(core_intrinsics)]
#![feature(cell_update)]
#![feature(generators, generator_trait)]
// #![feature(coroutines)]
// #![feature(offset_of)]

#[macro_use]
extern crate klib;

#[macro_use]
extern crate syscall;

pub mod malloc;
mod bit_trie;

#[cfg(test)]
mod bit_trie_test;

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
