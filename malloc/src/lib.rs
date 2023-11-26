#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]
#![feature(asm_const)]
#![feature(core_intrinsics)]
#![feature(cell_update)]
#![feature(coroutines, coroutine_trait)]
#![feature(offset_of)]

extern crate alloc;
extern crate klib;
extern crate syscall;

mod bit_trie;
pub mod malloc;

#[cfg(test)]
mod bit_trie_test;

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
