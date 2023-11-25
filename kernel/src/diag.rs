#[macro_export]
macro_rules! printk {
    ($fmt:expr $(,$args:expr)*) => {
        use klib::fmt::FormattedWriter;
        klib::make_args!($($args),*).format(&mut $crate::console::ConsoleWriter, $fmt)
    }
}

#[macro_export]
macro_rules! kpanic {
    ($fmt:expr $(,$args:expr)*) => {
        use klib::fmt::FormattedWriter;
        klib::make_args!($($args),*).format(&mut $crate::console::ConsoleWriter, $fmt);
        loop {}
    }
}
