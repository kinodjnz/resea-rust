use super::uart;
use crate::arch::console::ArchConsole;

pub struct Console;

impl ArchConsole for Console {
    fn print_char(ch: u8) {
        uart::tx(ch)
    }

    fn read_char() -> Option<u8> {
        uart::rx()
    }
}
