use crate::syscall;
use core::mem;
use klib::buf_writer::BufWriter;

pub fn print_error_impl<const N: usize>(format: &[u8], err: u32) {
    let mut buf = [mem::MaybeUninit::uninit(); N];
    let mut writer = BufWriter::new(&mut buf);
    klib::buf_fmt!(&mut writer, format, err);
    syscall::console_write(writer.as_slice());
}

#[macro_export]
macro_rules! print_error {
    ($message:expr, $err:expr) => {
        $crate::error::print_error_impl::<{ $message.len() + 8 }>($message, $err)
    };
}
