pub trait ArchIrq {
    fn enable_irq(irq: u32);
    fn disable_irq(irq: u32);
}

#[allow(dead_code)]
pub fn enable_irq(irq: u32) {
    <super::utilize::irq::Irq as ArchIrq>::enable_irq(irq);
}

#[allow(dead_code)]
pub fn disable_irq(irq: u32) {
    <super::utilize::irq::Irq as ArchIrq>::disable_irq(irq);
}
