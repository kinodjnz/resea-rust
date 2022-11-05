pub use crate::arch::Task;
use crate::config;
use crate::error::Error;
use crate::macros::*;
use crate::zeroed_array;

#[repr(align(16))]
struct Tasks {
    tasks: [Task; config::NUM_TASKS],
}

static mut TASKS: Tasks = Tasks {
    tasks: zeroed_array!(Task, config::NUM_TASKS),
};

impl Tasks {
    fn get_task(tid: u32) -> &'static Task {
        unsafe { TASKS.tasks.get_unchecked(tid as usize) }
    }

    fn get_task_mut(tid: u32) -> &'static mut Task {
        unsafe { TASKS.tasks.get_mut_unchecked(tid as usize) }
    }
}

pub struct NoarchTask {
    pub tid: u32,
    pub state: TaskState,
}

#[derive(PartialEq)]
pub enum TaskState {
    Unused = 0,
    Runnable,
    Blocked,
}

pub trait KArchTask {
    fn arch_task_create(task: NoarchTask, pc: usize) -> Result<Task, Error>;
    fn arch_task_switch(prev: &mut Task, next: &Task);
}

pub trait GetNoarchTask {
    fn get_noarch_task(&self) -> &NoarchTask;
    fn get_noarch_task_mut(&mut self) -> &mut NoarchTask;
}

pub trait TaskOps {
    fn create(tid: u32) -> Result<Task, Error>;
}

impl TaskOps for Task {
    fn create(tid: u32) -> Result<Task, Error> {
        Task::arch_task_create(
            NoarchTask {
                tid,
                state: TaskState::Blocked,
            },
            27,
        )
    }
}

pub fn task_create(tid: u32) -> Result<(), Error> {
    if Tasks::get_task(tid).get_noarch_task().state != TaskState::Unused {
        return Err(Error::AlreadyExists);
    }
    let task = Task::create(tid)?;
    *Tasks::get_task_mut(tid) = task;

    // let task2 = Task::arch_task_create(NoarchTask { tid: 1, state: TaskState::Blocked }, 49)?;
    // Task::arch_task_switch(unsafe { &mut TASKS.tasks[tid as usize] }, &task2);
    Ok(())
}
