use crate::arch::*;
pub use crate::fmt::*;

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

#[macro_export]
macro_rules! make_args {
    ($arg1:expr $(,$args:expr)*) => {
        HCons { head: $arg1, tail: make_args!($($args),*) }
    };
    () => {
        HNil
    };
}

#[macro_export]
macro_rules! printk {
    ($fmt:expr, $($args:expr),*) => {
        make_args!($($args),*).format(&mut ConsoleWriter, $fmt)
    }
}
