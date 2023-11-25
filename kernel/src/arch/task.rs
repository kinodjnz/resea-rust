pub use crate::arch::utilize::task::Task;
use crate::task::TaskRef;
use klib::result::KResult;

pub trait ArchTask {
    fn arch_task_init(tid: u32, task: TaskRef, pc: u32, sp: u32) -> KResult<()>;
    fn arch_idle_task_entry_point() -> u32;
    fn arch_task_switch(prev: &Task, next: &Task);
    fn arch_switch_idle_task();
    fn init_current(task: TaskRef);
    fn current() -> TaskRef;
}
