use crate::arch::KArchIrq;
use klib::mmio;

const REG_INTERRUPT_ENABLE: *mut u32 = 0x3000_4000 as *mut u32;
//const REG_INTERRUPT_STATUS: *mut u32 = 0x3000_4004 as *mut u32;

pub fn init() {
    mmio::writev(REG_INTERRUPT_ENABLE, 0);
}

pub struct Irq;

impl KArchIrq for Irq {
    fn enable_irq(irq: u32) {
        mmio::writev(
            REG_INTERRUPT_ENABLE,
            mmio::readv(REG_INTERRUPT_ENABLE) | (1 << irq),
        );
    }

    fn disable_irq(irq: u32) {
        mmio::writev(
            REG_INTERRUPT_ENABLE,
            mmio::readv(REG_INTERRUPT_ENABLE) & !(1 << irq),
        );
    }
}
