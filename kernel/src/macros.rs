pub use crate::console::ConsoleWriter;
pub use core::arch::asm;
pub use core::mem;
pub use klib::macros::*;

#[macro_export]
macro_rules! printk {
    ($fmt:expr $(,$args:expr)*) => {
        make_args!($($args),*).format(&mut ConsoleWriter, $fmt)
    }
}

#[macro_export]
macro_rules! kpanic {
    ($fmt:expr $(,$args:expr)*) => {
        make_args!($($args),*).format(&mut ConsoleWriter, $fmt);
        loop {}
    }
}
