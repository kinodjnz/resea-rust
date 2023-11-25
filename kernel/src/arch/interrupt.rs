pub trait ArchInterrupt {
    fn enable_interrupt();
    fn disable_interrupt();
}

#[allow(dead_code)]
pub fn enable_interrupt() {
    <super::utilize::interrupt::Interrupt as ArchInterrupt>::enable_interrupt();
}

pub fn disable_interrupt() {
    <super::utilize::interrupt::Interrupt as ArchInterrupt>::disable_interrupt();
}
