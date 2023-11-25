use super::csr;
use super::timer;
use crate::boot::kmain;
use klib::{local_address_of, mmio};

fn init_bss() {
    let bss_start = local_address_of!("__bss_start");
    let bss_end = local_address_of!("__bss_end");
    mmio::mzero_align4(bss_start as *mut u32, bss_end as *const u32);
}

#[no_mangle]
pub extern "C" fn cramp32_init() {
    init_bss();
    csr::init_csr();
    timer::init();
    kmain();
}
