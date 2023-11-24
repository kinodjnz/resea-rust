#[cfg(all(target_arch = "riscv32", feature = "cramp32"))]
pub mod cramp32 {
    pub mod memcpy;
    pub mod memset;
}
