use core::ptr::{read_volatile, write_volatile};

pub fn readv<T>(addr: *const T) -> T {
    unsafe { read_volatile(addr) }
}

pub fn writev<T>(addr: *mut T, value: T) {
    unsafe {
        write_volatile(addr, value);
    }
}

#[allow(unused)]
pub fn aligned<T, const N: usize>(addr: *const T) -> *const T {
    let p = addr as *const u8 as usize;
    (p & !(N - 1)) as *const T
}
