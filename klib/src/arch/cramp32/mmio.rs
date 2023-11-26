#[macro_export]
macro_rules! local_address_of {
    ($symbol: expr) => {
        {
            let mut temp_addr: u32;
            #[allow(unused_unsafe)]
            unsafe {
                core::arch::asm!(concat!("lla {0}, ", $symbol), out(reg) temp_addr);
            }
            temp_addr
        }
    }
}
