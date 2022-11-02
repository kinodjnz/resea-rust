use crate::start::kmain;
use core::arch::asm;
use core::mem::zeroed;
use core::ptr::write_volatile;

fn init_bss() {
    extern "C" {
        static mut __bss_start: u32;
        static mut __bss_end: u32;
    }
    unsafe {
        let mut p: *mut u32 = &mut __bss_start;
        let q: *mut u32 = &mut __bss_end;

        while p < q {
            write_volatile(p, zeroed());
            p = p.offset(1);
        }
    }
}

fn enable_interrupt() {
    unsafe {
        asm!("csrsi mstatus, 8");
    }
}

fn enable_machine_external_interrupt() {
    let mut mask: u32;
    unsafe {
        asm!("li   {0}, 0x800", out(reg) mask);
        asm!("csrs mie, {0}", in(reg) mask);
    }
}

fn init_csr() {
    let mut intr_handler: u32;
    unsafe {
        asm!("la   {0}, intr_handler", out(reg) intr_handler);
        asm!("csrw mtvec, {0}", in(reg) intr_handler);
    }
    enable_interrupt();
    enable_machine_external_interrupt();
}

#[no_mangle]
pub extern "C" fn cramp32_init() -> ! {
    init_bss();
    init_csr();
    kmain();
}
