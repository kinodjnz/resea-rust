use super::irq;
use super::macros::*;
use super::timer;
use crate::boot::kmain;
use klib::mmio;

fn init_bss() {
    let bss_start = local_address_of!("__bss_start");
    let bss_end = local_address_of!("__bss_end");
    mmio::mzero_align4(bss_start as *mut u32, bss_end as *const u32);
}

fn enable_interrupt() {
    cramp32_csrsi!("mstatus", 0x80);
}

fn enable_machine_external_and_timer_interrupt() {
    cramp32_csrsi!("mie", 0x880);
}

fn init_csr() {
    let intr_handler_ptr = local_address_of!("intr_handler");
    cramp32_csrw!("mtvec", intr_handler_ptr);
    irq::init();
    enable_interrupt();
    enable_machine_external_and_timer_interrupt();
}

#[no_mangle]
pub extern "C" fn cramp32_init() {
    init_bss();
    init_csr();
    timer::init();
    kmain();
}
