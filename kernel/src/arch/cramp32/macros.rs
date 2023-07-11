pub use crate::{cramp32_csrs, cramp32_csrsi, cramp32_csrw};
pub use core::arch::asm;

#[macro_export]
macro_rules! cramp32_csrsi {
    ($reg: expr, $imm: expr $(,)?) => {
        if $imm < 32 {
            unsafe {
                asm!(concat!("csrsi ", $reg, ", ", $imm));
            }
        } else {
            unsafe {
                asm!(concat!("csrs ", $reg, ", {0}"), in(reg) $imm);
            }
        }
    }
}

#[macro_export]
macro_rules! cramp32_csrs {
    ($reg: expr, $val: expr $(,)?) => {
        unsafe {
            asm!(concat!("csrs ", $reg, ", {0}"), in(reg) $val);
        }
    }
}

#[macro_export]
macro_rules! cramp32_csrw {
    ($reg: expr, $val: expr $(,)?) => {
        unsafe {
            asm!(concat!("csrw ", $reg, ", {0}"), in(reg) $val);
        }
    }
}
