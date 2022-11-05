use core::mem::MaybeUninit;

pub struct HNil;

pub struct HCons<H, T> {
    pub head: H,
    pub tail: T,
}

pub trait Display {
    fn fmt(&self, writer: &mut dyn Write);
}

pub trait Write {
    fn write_char(&mut self, ch: u8);
}

pub trait FormattedWriter<Args> {
    fn format(&self, writer: &mut dyn Write, format: &[u8]);
}

impl FormattedWriter<HNil> for HNil {
    fn format(&self, writer: &mut dyn Write, format: &[u8]) {
        for &ch in format {
            writer.write_char(ch);
        }
    }
}

impl<T, Args, ArgList> FormattedWriter<HCons<T, Args>> for HCons<T, ArgList>
where
    ArgList: FormattedWriter<Args>,
    T: Display,
{
    fn format(&self, writer: &mut dyn Write, format: &[u8]) {
        let mut i = 0;
        while i < format.len() {
            let ch = format[i];
            i += 1;
            if ch == b'{' {
                break;
            }
            writer.write_char(ch);
        }
        while i < format.len() {
            let ch = format[i];
            i += 1;
            if ch == b'}' {
                self.head.fmt(writer);
                self.tail.format(writer, format);
                return;
            }
        }
    }
}

impl Display for u32 {
    fn fmt(&self, writer: &mut dyn Write) {
        let mut buf: [MaybeUninit<u8>; 10] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut x = *self;
        let mut i: usize = 0;
        loop {
            let d = (x % 10) as u8;
            x = x / 10;
            unsafe { buf.get_unchecked_mut(i).write(b'0' + d) };
            i += 1;
            if x == 0 {
                break;
            }
        }
        while i > 0 {
            i -= 1;
            writer.write_char(unsafe { buf.get_unchecked(i).assume_init_read() });
        }
    }
}

impl Display for usize {
    fn fmt(&self, writer: &mut dyn Write) {
        let mut buf: [MaybeUninit<u8>; 8] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut x = *self;
        writer.write_char(b'0');
        writer.write_char(b'x');
        for i in 0..8 {
            let d = (x & 15) as u8;
            x = x >> 4;
            let ch = if d >= 10 { b'a' + d - 10 } else { b'0' + d };
            unsafe { buf.get_unchecked_mut(7 - i).write(ch) };
        }
        for i in 0..8 {
            writer.write_char(unsafe { buf.get_unchecked(i).assume_init_read() });
        }
    }
}
