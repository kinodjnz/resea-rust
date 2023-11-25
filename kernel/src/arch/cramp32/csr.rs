use super::irq;
use klib::local_address_of;

#[allow(unused_macros)]
macro_rules! cramp32_csrsi {
    ($reg: expr, $imm: expr $(,)?) => {
        if $imm < 32 {
            unsafe {
                core::arch::asm!(concat!("csrsi ", $reg, ", ", $imm));
            }
        } else {
            unsafe {
                core::arch::asm!(concat!("csrs ", $reg, ", {0}"), in(reg) $imm);
            }
        }
    }
}

#[allow(unused_macros)]
macro_rules! cramp32_csrs {
    ($reg: expr, $val: expr $(,)?) => {
        unsafe {
            core::arch::asm!(concat!("csrs ", $reg, ", {0}"), in(reg) $val);
        }
    }
}

#[allow(unused_macros)]
macro_rules! cramp32_csrw {
    ($reg: expr, $val: expr $(,)?) => {
        unsafe {
            core::arch::asm!(concat!("csrw ", $reg, ", {0}"), in(reg) $val);
        }
    }
}

fn enable_interrupt() {
    cramp32_csrsi!("mstatus", 0x80);
}

fn enable_machine_external_and_timer_interrupt() {
    cramp32_csrsi!("mie", 0x880);
}

pub fn init_csr() {
    let intr_handler_ptr = local_address_of!("intr_handler");
    cramp32_csrw!("mtvec", intr_handler_ptr);
    irq::init();
    enable_interrupt();
    enable_machine_external_and_timer_interrupt();
}
