use super::interrupt;
use super::irq;
use klib::local_address_of;

#[macro_export]
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

#[macro_export]
macro_rules! cramp32_csrs {
    ($reg: expr, $val: expr $(,)?) => {
        unsafe {
            core::arch::asm!(concat!("csrs ", $reg, ", {0}"), in(reg) $val);
        }
    }
}

#[macro_export]
macro_rules! cramp32_csrci {
    ($reg: expr, $imm: expr $(,)?) => {
        if $imm < 32 {
            unsafe {
                core::arch::asm!(concat!("csrci ", $reg, ", ", $imm));
            }
        } else {
            unsafe {
                core::arch::asm!(concat!("csrc ", $reg, ", {0}"), in(reg) $imm);
            }
        }
    }
}

#[macro_export]
macro_rules! cramp32_csrc {
    ($reg: expr, $val: expr $(,)?) => {
        unsafe {
            core::arch::asm!(concat!("csrc ", $reg, ", {0}"), in(reg) $val);
        }
    }
}

#[macro_export]
macro_rules! cramp32_csrw {
    ($reg: expr, $val: expr $(,)?) => {
        unsafe {
            core::arch::asm!(concat!("csrw ", $reg, ", {0}"), in(reg) $val);
        }
    }
}

pub fn init_csr() {
    let intr_handler_ptr = local_address_of!("intr_handler");
    cramp32_csrw!("mtvec", intr_handler_ptr);
    irq::init();
    interrupt::init_interrupt();
    interrupt::enable_machine_external_and_timer_interrupt();
}
