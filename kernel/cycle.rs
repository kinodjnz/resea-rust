use super::mmio::readv;
use core::arch::asm;

const REG_CONFIG_CLOCK_HZ: *mut u32 = 0x4000_0004 as *mut u32;
static mut CLOCK_HZ: u32 = 0;

#[allow(dead_code)]
fn read_cycle() -> u64 {
    let mut l: u32;
    let mut h: u32;
    let mut hv: u32;
    loop {
        unsafe {
            asm!("rdcycleh {0}", out(reg) h);
            asm!("rdcycle  {0}", out(reg) l);
            asm!("rdcycleh {0}", out(reg) hv);
        }
        if h == hv {
            break;
        }
    }
    ((h as u64) << 32) | (l as u64)
}

#[allow(dead_code)]
pub fn clock_hz() -> u32 {
    unsafe { CLOCK_HZ }
}

#[allow(dead_code)]
pub fn wait(cycles: u32) {
    let start: u64 = read_cycle();
    while read_cycle() - start < (cycles as u64) {}
}

pub fn init() {
    unsafe {
        CLOCK_HZ = readv(REG_CONFIG_CLOCK_HZ);
    }
}
