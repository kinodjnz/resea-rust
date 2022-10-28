use core::mem::zeroed;
use core::ptr::write_volatile;

pub fn init_bss() {
    extern "C" {
        static mut __bss_start: u32;
        static mut __bss_end: u32;
    }
    unsafe {
        let mut p: *mut u32 = &mut __bss_start;
        let q: *mut u32 = &mut __bss_end;

        while p < q {
            write_volatile(p, zeroed());
            p = p.offset(1);
        }
    }
}

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    init_bss();
    super::cycle::init();
    super::main();
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
