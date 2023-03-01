use super::irq;
use super::macros::*;
use super::timer;
use crate::boot::kmain;
use klib::mmio;

fn init_bss() {
    extern "C" {
        static mut __bss_start: u32;
        static __bss_end: u32;
    }
    unsafe {
        mmio::mzero_align4(&mut __bss_start, &__bss_end);
    }
}

fn enable_interrupt() {
    cramp32_csrsi!("mstatus", 8);
}

fn enable_machine_external_and_timer_interrupt() {
    cramp32_csrsi!("mie", 0x880);
}

fn init_csr() {
    extern "C" {
        fn intr_handler();
    }
    cramp32_csrw!("mtvec", intr_handler as u32);
    irq::init();
    enable_interrupt();
    enable_machine_external_and_timer_interrupt();
}

#[no_mangle]
pub extern "C" fn cramp32_init() -> ! {
    init_bss();
    init_csr();
    timer::init();
    kmain();
}
