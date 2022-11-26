#![no_std]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(try_trait_v2)]

#[macro_use]
mod macros;

mod cycle;
//mod gpio;
//mod loader;
mod mmio;
//mod sdc;
mod arch;
mod boot;
mod config;
mod console;
mod fmt;
mod list;
mod result;
mod syscall;
mod task;

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
#[link_section = ".panic_info"]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
