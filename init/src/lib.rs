#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]
#![feature(asm_const)]
#![feature(coroutines, coroutine_trait)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_uninit_array)]
#![feature(sync_unsafe_cell)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate klib;
extern crate syscall;

mod generator;
pub mod init;

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
    loop {}
}
