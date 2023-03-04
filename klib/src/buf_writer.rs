use crate::fmt::Write;
use core::mem;
use core::ptr;

pub struct BufWriter<'a> {
    buf: &'a mut [mem::MaybeUninit<u8>],
    pos: usize,
}

impl<'a> BufWriter<'a> {
    pub fn new(buf: &'a mut [mem::MaybeUninit<u8>]) -> BufWriter<'a> {
        BufWriter { buf, pos: 0 }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            &*ptr::slice_from_raw_parts(
                mem::MaybeUninit::slice_assume_init_ref(self.buf).as_ptr(),
                self.pos,
            )
        }
    }
}

impl<'a> Write for BufWriter<'a> {
    fn write_char(&mut self, ch: u8) {
        if self.pos < self.buf.len() {
            unsafe {
                self.buf.get_unchecked_mut(self.pos).write(ch);
            }
            self.pos += 1;
        }
    }
}
