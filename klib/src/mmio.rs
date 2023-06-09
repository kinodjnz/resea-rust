use core::mem;
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

pub fn mzero_align4<T>(p: *mut T, q: *const T) {
    unsafe {
        let mut p = p as *mut u32;
        let q = q as *const u32;
        while (p as *const u32) != q {
            writev(p, 0);
            p = p.add(1);
        }
    }
}

#[allow(unused)]
pub const fn size_of_aligned4<T>() -> usize {
    let size = mem::size_of::<T>();
    if size % 4 != 0 {
        panic!("hoge");
    }
    size
}
