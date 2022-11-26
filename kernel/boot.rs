use crate::cycle;
use crate::macros::*;
use crate::task;
use crate::syscall;

pub fn kmain() -> ! {
    printk!(b"\nBooting Resea v0.0.1\n");
    cycle::init();
    if task::get_task_pool()
        .create_user_task(task::INIT_TASK_TID, (init_task as *const ()) as usize)
        .is_err()
    {
        printk!(b"create init task failed");
    }
    if task::get_task_pool()
        .create_user_task(2, (worker_task as *const ()) as usize)
        .is_err()
    {
        printk!(b"create worker task failed");
    }
    if task::get_task_pool().create_idle_task().is_err() {
        printk!(b"create idle task failed\n");
    }
    loop {}
}

pub fn init_task() -> ! {
    printk!(b"init task started\n");
    loop {
        printk!(b"Hello, Resea\n");
        cycle::wait(cycle::clock_hz());
    }
}

pub fn worker_task() -> ! {
    cycle::wait(cycle::clock_hz() / 2);
    printk!(b"worker task started\n");
    loop {
        syscall::console_write(b"Hello, RISC-V\n");
        syscall::nop();
        cycle::wait(cycle::clock_hz());
    }
}
