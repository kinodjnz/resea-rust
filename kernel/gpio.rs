use super::mmio::writev;

static mut REG_GPIO_OUT: *mut u32 = 0x3000_0000 as *mut u32;

pub fn out(value: u32) {
    unsafe {
        writev(REG_GPIO_OUT, value);
    }
}
