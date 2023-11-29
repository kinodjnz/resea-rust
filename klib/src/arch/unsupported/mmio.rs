#[macro_export]
macro_rules! local_address_of {
    ($symbol: expr) => {
        unsafe {
            extern "C" {
                #[link_name = $symbol]
                static mut x: u32;
            }
            (&mut x as *mut u32) as u32
        }
    };
}
