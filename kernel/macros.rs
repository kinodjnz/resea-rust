pub use crate::console::ConsoleWriter;
pub use crate::fmt::*;

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
