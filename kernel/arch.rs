pub mod cramp32;

pub trait KArchConsole {
    fn print_char(ch: u8);
    fn read_char() -> Option<u8>;
}

pub trait KArchIrq {
    fn enable_irq(irq: u32);
    fn disable_irq(irq: u32);
}

#[cfg(feature = "cramp32")]
pub type ArchConsole = crate::arch::cramp32::console::Console;

#[allow(unused)]
#[cfg(feature = "cramp32")]
pub type ArchIrq = crate::arch::cramp32::irq::Irq;

#[cfg(feature = "cramp32")]
pub type Task<'t> = crate::arch::cramp32::task::Cramp32Task<'t>;
