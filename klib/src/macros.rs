pub use crate::buf_writer::BufWriter;
pub use crate::fmt::*;
pub use core::arch::asm;

#[macro_export]
macro_rules! make_args {
    ($arg1:expr $(,$args:expr)*) => {
        HCons { head: $arg1, tail: make_args!($($args),*) }
    };
    () => {
        HNil
    };
}

#[macro_export]
macro_rules! buf_fmt {
    ($buf:expr, $fmt:expr $(,$args:expr)*) => {
        make_args!($($args),*).format($buf, $fmt)
    }
}

#[macro_export]
macro_rules! local_address_of {
    ($symbol: expr) => {
        {
            let mut temp_addr: usize;
            #[allow(unused_unsafe)]
            unsafe {
                asm!(concat!("lla {0}, ", $symbol), out(reg) temp_addr);
            }
            temp_addr
        }
    }
}
