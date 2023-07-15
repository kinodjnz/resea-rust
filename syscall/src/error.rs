use crate::syscall;
use core::mem;
use klib::macros::*;
use klib::*;

pub fn print_error<const N: usize>(format: &[u8], err: u32) {
    let mut buf = [mem::MaybeUninit::uninit(); N];
    let mut writer = BufWriter::new(&mut buf);
    buf_fmt!(&mut writer, format, err);
    syscall::console_write(writer.as_slice());
}
