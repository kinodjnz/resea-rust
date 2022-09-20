use super::mmio::{readv, writev};

const REG_UART_STATUS: *mut u32 = 0x3000_1000 as *mut u32;
const REG_UART_DATA: *mut u32 = 0x3000_1004 as *mut u32;

pub fn tx(value: u8) {
    while ((readv(REG_UART_STATUS)) & 4) != 0 {}
    writev(REG_UART_DATA, value as u32);
}

pub fn puts(s: &[u8]) {
    for c in s.iter() {
        tx(*c);
    }
}

pub fn print(mut x: u32) {
    tx(b'0');
    tx(b'x');
    let mut buf: [u8; 9] = [0; 9];
    for i in 0..8 {
        let d = (x % 16) as u8;
        buf[7 - i] = if d < 10 { b'0' + d } else { b'A' - 10 + d };
        x = x / 16;
    }
    buf[8] = b'\0';
    puts(&buf);
}
