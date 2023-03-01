use crate::syscall;
use klib::cycle;

pub fn init_task() -> ! {
    syscall::console_write(b"init task started\n");
    loop {
        syscall::console_write(b"Hello, Resea\n");
        cycle::wait(cycle::clock_hz());
    }
}
