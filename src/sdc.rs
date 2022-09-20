use super::cycle;
use super::mmio::{readv, writev};

const REG_SDC_SETTING: *mut u32 = 0x3000_3000 as *mut u32;
const REG_SDC_CONTROL: *mut u32 = 0x3000_3004 as *mut u32;
const REG_SDC_STATUS: *mut u32 = 0x3000_3004 as *mut u32;
const REG_SDC_COMMAND: *mut u32 = 0x3000_3008 as *mut u32;
const REG_SDC_RESPONSE: *mut u32 = 0x3000_3008 as *mut u32;
const REG_SDC_DATA: *mut u32 = 0x3000_300c as *mut u32;

static mut BLOCK_ACCESS: bool = false;

fn cmd(control: u32, command: u32) -> (u32, u32) {
    writev(REG_SDC_COMMAND, command);
    writev(REG_SDC_CONTROL, control);
    if (control & 0x0f) == 0 {
        return (0, 0);
    }
    let mut s: u32 = 0;
    while (s & 1) == 0 {
        s = readv(REG_SDC_STATUS);
    }
    let r = readv(REG_SDC_RESPONSE);
    (if s == 1 { 0 } else { s }, r)
}

fn acmd(control: u32, command: u32, rca: u32) -> (u32, u32) {
    let (s, r) = cmd(0x0000_0b71, rca << 16); // CMD55
    if s != 0 {
        return (s, r);
    }
    let (s, r) = cmd(control, command);
    if s != 0 {
        return (0x0000_0200 + s, r);
    }
    (0, r)
}

pub fn read_sector(sector_address: u32, buf: *mut u32) -> u32 {
    let address: u32 = unsafe {
        if BLOCK_ACCESS {
            sector_address
        } else {
            sector_address * 512
        }
    };
    let (s, _r) = cmd(0x0000_1911, address); // CMD17 / R1
    if s != 0 {
        return s;
    }
    let mut s = 0;
    while (s & 0x0001_0000) == 0 {
        s = readv(REG_SDC_STATUS);
    }
    if s != 0x0001_0000 {
        return 0x0000_2000 + s;
    }
    for i in 0..128 {
        let x = readv(REG_SDC_DATA);
        unsafe {
            writev(buf.add(i), x);
        }
    }
    0
}

pub fn init_card() -> u32 {
    cycle::wait(50500000); // 電源が安定するまで 500ms 待つ
    writev(REG_SDC_SETTING, 0x8000_007d); // divider = 252 : 400kHz
    cycle::wait(20000); // 最低 74 sd clock (18648 clock) 待つ
    cmd(0x0000_0800, 0x0000_0000); // CMD0 / R-
    cycle::wait(101000); // 1ms 待つ
    cmd(0x0000_0887, 0x0000_01aa); // CMD8 / R7
    cycle::wait(101000); // 1ms 待つ
    let mut initialized = false;
    for _i in 0..70 {
        let (s, r) = acmd(0x0000_0a93, 0x40ff_8000, 0); // ACMD41 / R3
        if s != 0 {
            return 1;
        }
        if (r & 0x8000_0000) != 0 {
            initialized = true;
            unsafe {
                BLOCK_ACCESS = (r & 0x4000_0000) != 0;
            }
            break;
        }
        cycle::wait(1010000); // 10ms 待つ
    }
    if !initialized {
        return 2;
    }
    let (s, _r) = cmd(0x0000_0822, 0x0000_0000); // CMD2 / R2
    if s != 0 {
        return 3;
    }
    // uart::print(r);
    // uart::puts(b" #cmd2\r\n");
    for _ in 0..3 {
        let _r = readv(REG_SDC_RESPONSE);
        // uart::print(r);
        // uart::puts(b" #cmd2\r\n");
    }
    let (s, r) = cmd(0x0000_0836, 0x0000_0000); // CMD3 / R6
    if s != 0 {
        return 4;
    }
    // uart::print(r);
    // uart::puts(b" #cmd3\r\n");
    let rca: u32 = r >> 16;
    // card identification mode ends here

    //writev(REG_SDC_SETTING, 0x8000_0001); // divider = 4 : 25.01MHz
    writev(REG_SDC_SETTING, 0x8000_0003); // divider = 8 : 12.00MHz
    cycle::wait(4000); // 念のため clock 安定化待ち

    // let (s, r) = cmd(0x0000_0892, rca << 16); // CMD9 / R2
    // if s != 0 {
    //     uart::print(s);
    //     uart::puts(b" #cmd9 s\r\n");
    //     return;
    // }
    // uart::print(r);
    // uart::puts(b" #cmd9\r\n");
    // for _ in 0..3 {
    //     let r = readv(REG_SDC_RESPONSE);
    //     uart::print(r);
    //     uart::puts(b" #cmd9\r\n");
    // }
    let (s, _r) = cmd(0x0000_0875, rca << 16); // CMD7 / R1b
    if s != 0 {
        return 5;
    }
    let (s, _r) = cmd(0x0000_0901, 512); // CMD16 / R1
    if s != 0 {
        return 6;
    }
    let (s, _r) = acmd(0x0000_0861, 2, rca); // ACMD6 / R1
    if s != 0 {
        return 7;
    }
    0
}
