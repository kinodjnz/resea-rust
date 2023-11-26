#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
pub mod cramp32 {
    pub mod mmio;
}

#[cfg(not(all(target_arch = "riscv32", feature = "cramp32")))]
pub mod unsupported {
    pub mod mmio;
}
