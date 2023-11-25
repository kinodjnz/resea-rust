#![no_std]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(try_trait_v2)]
#![feature(cell_update)]

extern crate klib;

//mod gpio;
//mod loader;
//mod sdc;
mod arch;
mod boot;
mod config;
mod console;
mod diag;
mod ipc;
mod syscall;
mod task;

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
#[link_section = ".panic_info"]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
