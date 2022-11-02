#![no_std]

#[macro_use]
mod macros;

mod cycle;
mod gpio;
//mod loader;
mod mmio;
//mod sdc;
mod arch;
mod console;
mod fmt;
mod start;
mod syscall;

use macros::*;

fn putx(x: u32) {
    printk!(b"x is {}", x);
    printk!(b"*2: {}, +4: {}", x * 2, x + 4);
    printk!(b"{} again", x);
}

fn main() {
    // let s = sdc::init_card();
    // uart::print(s);
    // uart::puts(b" sd\r\n");
    // let s = loader::load_kernel();
    // uart::print(s);
    // uart::puts(b" ld\r\n");
    let mut led_out: u32 = 1;
    loop {
        //Console::puts("Hello, RISC-V\r\n");
        putx(led_out);
        gpio::out(led_out);
        led_out = (led_out << 1) | ((led_out >> 7) & 1);
        cycle::wait(cycle::clock_hz() >> 1);
    }
}

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
#[link_section = ".panic_info"]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
