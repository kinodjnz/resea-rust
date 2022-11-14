use crate::macros::*;

pub fn kmain() -> ! {
    printk!(b"\nBooting CResea v0.1\n");
    if crate::task::get_task_pool().create_idle_task().is_ok() {
        printk!(b"task_create success\n");
    }
    loop {}
}
