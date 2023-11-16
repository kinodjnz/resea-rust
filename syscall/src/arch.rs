#[cfg(target_arch = "riscv32")]
pub mod riscv32;

#[cfg(target_arch = "riscv32")]
pub use riscv32::syscall;

#[cfg(not(target_arch = "riscv32"))]
pub mod unsupported;

#[cfg(not(target_arch = "riscv32"))]
pub use unsupported::syscall;
