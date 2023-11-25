pub trait ArchConsole {
    fn print_char(ch: u8);
    fn read_char() -> Option<u8>;
}

pub fn print_char(ch: u8) {
    <super::utilize::console::Console as ArchConsole>::print_char(ch);
}
pub fn read_char() -> Option<u8> {
    <super::utilize::console::Console as ArchConsole>::read_char()
}
