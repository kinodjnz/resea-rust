use super::uart;
use crate::arch::KArchConsole;

pub struct Console;

impl KArchConsole for Console {
    fn print_char(ch: u8) {
        uart::tx(ch)
    }

    fn read_char() -> Option<u8> {
        uart::rx()
    }
}
