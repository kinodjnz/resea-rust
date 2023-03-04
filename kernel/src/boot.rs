use crate::macros::*;
use crate::task;

pub fn kmain() -> ! {
    printk!(b"\nBooting Resea/Rust v0.0.1\n");
    extern "C" {
        fn init_task();
    }
    if task::get_task_pool()
        .create_user_task(task::INIT_TID, (init_task as *const ()) as usize)
        .is_err()
    {
        printk!(b"create init task failed");
    }
    if task::get_task_pool().create_idle_task().is_err() {
        printk!(b"create idle task failed\n");
    }
    loop {}
}
