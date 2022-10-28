use core::ptr::{read_volatile, write_volatile};

pub fn readv<T>(addr: *const T) -> T {
    unsafe { read_volatile(addr) }
}

pub fn writev<T>(addr: *mut T, value: T) {
    unsafe {
        write_volatile(addr, value);
    }
}
