use crate::arch::console as arch;
use klib::fmt::Write;

pub struct Console;

impl Console {
    #[allow(dead_code)]
    pub fn puts(s: &[u8]) {
        for c in s.iter() {
            arch::print_char(*c);
        }
    }

    #[allow(dead_code)]
    pub fn print_char(ch: u8) {
        arch::print_char(ch);
    }

    #[allow(dead_code)]
    pub fn read_char() -> Option<u8> {
        arch::read_char()
    }
}

pub struct ConsoleWriter;

impl Write for ConsoleWriter {
    fn write_char(&mut self, ch: u8) {
        Console::print_char(ch);
    }
}
