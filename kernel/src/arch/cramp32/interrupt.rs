use crate::arch::interrupt::ArchInterrupt;
use crate::{cramp32_csrci, cramp32_csrsi};

pub fn init_interrupt() {
    cramp32_csrsi!("mstatus", 0x80);
}

pub fn enable_machine_external_and_timer_interrupt() {
    cramp32_csrsi!("mie", 0x880);
}

pub struct Interrupt;

impl ArchInterrupt for Interrupt {
    fn enable_interrupt() {
        cramp32_csrsi!("mstatus", 8);
    }

    fn disable_interrupt() {
        cramp32_csrci!("mstatus", 8);
    }
}
