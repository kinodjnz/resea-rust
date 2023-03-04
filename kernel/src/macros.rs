pub use crate::console::ConsoleWriter;
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

#[macro_export]
macro_rules! zeroed_array {
    ($elem: ty, $size: expr) => {
        unsafe {
            mem::transmute::<[u32; mem::size_of::<$elem>() * $size / 4], [$elem; $size]>(
                [0; mem::size_of::<$elem>() * $size / 4],
            )
        }
    };
}

#[macro_export]
macro_rules! zeroed_const {
    ($elem: ty) => {
        unsafe {
            mem::transmute::<[u32; mem::size_of::<$elem>() / 4], $elem>(
                [0; mem::size_of::<$elem>() / 4],
            )
        }
    };
}
