#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
mod cramp32 {
    pub mod console;
    mod csr;
    mod init;
    mod interrupt;
    pub mod irq;
    pub mod task;
    mod timer;
    mod uart;
}

pub trait KArchConsole {
    fn print_char(ch: u8);
    fn read_char() -> Option<u8>;
}

pub trait KArchIrq {
    fn enable_irq(irq: u32);
    fn disable_irq(irq: u32);
}

#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
pub type ArchConsole = crate::arch::cramp32::console::Console;

#[allow(unused)]
#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
pub type ArchIrq = crate::arch::cramp32::irq::Irq;

#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
pub type Task = crate::arch::cramp32::task::Cramp32Task;
