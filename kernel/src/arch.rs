pub mod console;
pub mod interrupt;
pub mod irq;
pub mod task;

#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
mod cramp32 {
    pub mod console;
    mod csr;
    mod init;
    pub mod interrupt;
    pub mod irq;
    pub mod task;
    mod timer;
    mod trap;
    mod uart;
}

#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
mod utilize {
    pub use crate::arch::cramp32::console;
    pub use crate::arch::cramp32::interrupt;
    pub use crate::arch::cramp32::irq;
    pub use crate::arch::cramp32::task;
}
