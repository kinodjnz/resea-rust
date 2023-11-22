use crate::macros::*;
use crate::task::{self, TaskPool};

pub fn kmain() {
    printk!(b"\nBooting Resea/Rust v0.0.1\n");
    let init_task_ptr = local_address_of!("init_task");
    if task::get_task_pool()
        .create_user_task(task::INIT_TID, init_task_ptr)
        .is_err()
    {
        printk!(b"create init task failed");
    }
    if task::get_task_pool().create_idle_task().is_err() {
        printk!(b"create idle task failed\n");
    }
    TaskPool::switch_idle_task();
}
