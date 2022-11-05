use crate::mmio;

const REG_MTIMER_MTIME_LO: *mut u32 = 0x3000_2000 as *mut u32;
const REG_MTIMER_MTIME_HI: *mut u32 = 0x3000_2004 as *mut u32;
const REG_MTIMER_MTIMECMP_LO: *mut u32 = 0x3000_2008 as *mut u32;
const REG_MTIMER_MTIMECMP_HI: *mut u32 = 0x3000_200C as *mut u32;

const REG_CONFIG_CLOCK_HZ: *mut u32 = 0x4000_0004 as *mut u32;

static mut TIMER: MachineTimer = MachineTimer {
    next_tick: 0,
    tick_span: 0,
};

pub struct MachineTimer {
    next_tick: u64,
    tick_span: u32,
}

pub fn init() {
    unsafe {
        TIMER = MachineTimer::init();
    }
}

pub fn reload() {
    unsafe {
        TIMER.reload();
    }
}

impl MachineTimer {
    fn read_mtime() -> u64 {
        let mut mtime_lo;
        let mut mtime_hi;
        loop {
            mtime_hi = mmio::readv(REG_MTIMER_MTIME_HI);
            mtime_lo = mmio::readv(REG_MTIMER_MTIME_LO);
            if mtime_hi == mmio::readv(REG_MTIMER_MTIME_HI) {
                return ((mtime_hi as u64) << 32) | mtime_lo as u64;
            }
        }
    }

    pub fn init() -> Self {
        let clock_hz = mmio::readv(REG_CONFIG_CLOCK_HZ);
        let tick_span = clock_hz / 1000;
        let next_tick = Self::read_mtime() + tick_span as u64;
        MachineTimer {
            next_tick,
            tick_span,
        }
    }

    pub fn reload(&mut self) {
        self.next_tick = self.next_tick + self.tick_span as u64;
        // TODO compare next_tick and mtime
        mmio::writev(REG_MTIMER_MTIMECMP_HI, u32::MAX);
        mmio::writev(
            REG_MTIMER_MTIMECMP_LO,
            (self.next_tick & 0xffff_ffffu64) as u32,
        );
        mmio::writev(REG_MTIMER_MTIMECMP_HI, (self.next_tick >> 32) as u32);
    }
}
