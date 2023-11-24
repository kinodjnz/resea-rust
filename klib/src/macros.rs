#[macro_export]
macro_rules! make_args {
    ($arg1:expr $(,$args:expr)*) => {
        $crate::fmt::HCons { head: $arg1, tail: $crate::make_args!($($args),*) }
    };
    () => {
        $crate::fmt::HNil
    };
}

#[macro_export]
macro_rules! buf_fmt {
    ($buf:expr, $fmt:expr $(,$args:expr)*) => {
        use $crate::fmt::FormattedWriter;
        $crate::make_args!($($args),*).format($buf, $fmt)
    }
}

#[macro_export]
macro_rules! local_address_of {
    ($symbol: expr) => {
        {
            let mut temp_addr: usize;
            #[allow(unused_unsafe)]
            unsafe {
                core::arch::asm!(concat!("lla {0}, ", $symbol), out(reg) temp_addr);
            }
            temp_addr
        }
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
