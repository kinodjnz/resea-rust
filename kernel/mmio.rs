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
        while (p as *const u32) < q {
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

pub fn mzero_array<T, const N: usize>(p: &mut [T; N]) {
    mzero_align4(p.as_mut_ptr(), unsafe {
        p.as_ptr().add(mem::size_of_val(p))
    });
}

#[allow(unused)]
pub fn mzero<T, const N: usize>(p: *mut T) {
    mzero_align4(p, unsafe { p.add(1) });
}

pub fn memcpy_align4<T>(dst: *mut T, src: *const T, count: usize) {
    unsafe {
        let q = src.add(count) as *const u32;
        let mut dst = dst as *mut u32;
        let mut src = src as *const u32;
        while src < q {
            writev(dst, *src);
            dst = dst.add(1);
            src = src.add(1);
        }
    }
}
