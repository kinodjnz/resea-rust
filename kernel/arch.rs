pub mod cramp32;

pub trait KArchConsole {
    fn print_char(ch: u8);
    fn read_char() -> Option<u8>;
}

#[cfg(feature = "cramp32")]
pub type ArchConsole = crate::arch::cramp32::console::Console;
