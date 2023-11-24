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
macro_rules! cramp32_csrw {
    ($reg: expr, $val: expr $(,)?) => {
        unsafe {
            core::arch::asm!(concat!("csrw ", $reg, ", {0}"), in(reg) $val);
        }
    }
}
