use crate::arch::*;
use klib::fmt::Write;

pub struct Console;

impl Console {
    #[allow(dead_code)]
    pub fn puts(s: &[u8]) {
        for c in s.iter() {
            ArchConsole::print_char(*c);
        }
    }

    #[allow(dead_code)]
    pub fn print_char(ch: u8) {
        ArchConsole::print_char(ch);
    }

    #[allow(dead_code)]
    pub fn read_char() -> Option<u8> {
        ArchConsole::read_char()
    }
}

pub struct ConsoleWriter;

impl Write for ConsoleWriter {
    fn write_char(&mut self, ch: u8) {
        Console::print_char(ch);
    }
}
