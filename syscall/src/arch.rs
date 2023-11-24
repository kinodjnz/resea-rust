#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
pub mod cramp32 {
    pub mod syscall;
}

#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
pub use cramp32::syscall;

#[cfg(not(all(target_arch = "riscv32", feature = "cramp32")))]
pub mod unsupported {
    pub mod syscall;
}

#[cfg(not(all(target_arch = "riscv32", feature = "cramp32")))]
pub use unsupported::syscall;
