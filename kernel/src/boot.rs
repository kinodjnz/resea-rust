use crate::printk;
use crate::task::{self, TaskPool};
use klib::local_address_of;

pub fn kmain() {
    printk!(b"\nBooting Resea/Rust v0.0.1\n");
    if task::get_task_pool()
        .create_user_task(
            task::INIT_TID,
            local_address_of!("init_task"),
            local_address_of!("__init_task_stack_end"),
        )
        .is_err()
    {
        printk!(b"create init task failed");
    }
    if task::get_task_pool().create_idle_task().is_err() {
        printk!(b"create idle task failed\n");
    }
    TaskPool::switch_idle_task();
}
