pub use crate::arch::Task;
use crate::config;
use crate::error::Error;
use crate::macros::*;
use crate::zeroed_array;
use crate::list;
use crate::list::ContainerAdapter;

const TASK_PRIORITY_MAX: u32 = 8;
const TASK_TIME_SLICE: i32 = 10; // should meet timer intr cycle

type TaskList = [Task; config::NUM_TASKS as usize];

#[repr(align(16))]
pub struct TaskPool {
    tasks: TaskList,
    current_tid: u32,
    runqueues: [list::ListElement<u32>; TASK_PRIORITY_MAX as usize],
}

static mut TASK_POOL: TaskPool = TaskPool {
    tasks: zeroed_array!(Task, config::NUM_TASKS as usize),
    current_tid: 0,
    runqueues: zeroed_array!(list::ListElement<u32>, TASK_PRIORITY_MAX as usize),
};

trait TaskListOps {
    fn task<'t>(&self, tid: u32) -> &'t Task;
    fn task_mut(&mut self, tid: u32) -> &mut Task;
}

impl TaskListOps for TaskList {
    fn task<'t>(&self, tid: u32) -> &'t Task {
        unsafe {
            &*(self.get_unchecked(tid as usize) as *const Task)
        }
    }

    fn task_mut(&mut self, tid: u32) -> &mut Task {
        unsafe {
            self.get_unchecked_mut(tid as usize)
        }
    }
}

impl list::ElementAdapter<RunqueueTag, u32> for Task {
    fn element(&self) -> &list::ListElement<u32> {
        &self.noarch().runqueue_link
    }

    fn element_mut(&mut self) -> &mut list::ListElement<u32> {
        &mut self.noarch_mut().runqueue_link
    }

    fn self_pointer(&self) -> u32 {
        self.noarch().tid
    }
}

impl list::ContainerAdapter<u32, Task> for TaskList {
    fn deref_pointer<'t>(&self, p: u32) -> &'t Task {
        self.task(p)
    }

    fn deref_pointer_mut(&mut self, p: u32) -> &mut Task {
        self.task_mut(p)
    }

    fn as_mut(&mut self, t: &Task) -> &mut Task {
        unsafe {
            &mut *(t as *const Task as *mut Task)
        }
    }

    fn as_mut2(&mut self, t1: &Task, t2: &Task) -> (&mut Task, &mut Task) {
        unsafe {
            (&mut *(t1 as *const Task as *mut Task), &mut *(t2 as *const Task as *mut Task))
        }
    }
}

struct RunqueueTag;

impl TaskPool {
    fn get_task<'t>(&self, tid: u32) -> &'t Task {
        self.tasks.task(tid)
    }

    fn get_task_mut(&mut self, tid: u32) -> &mut Task {
        self.tasks.task_mut(tid)
    }

    fn active_iter(&self) -> ActiveIter {
        ActiveIter { tid: 0, task_pool: self }
    }

    fn current_mut(&mut self) -> &mut Task {
        unsafe {
            self.tasks.get_unchecked_mut(self.current_tid as usize)
        }
    }

    fn element_adapter_of(task: &Task) -> &dyn list::ElementAdapter<RunqueueTag, u32> {
        task as &dyn list::ElementAdapter<RunqueueTag, u32>
    }

    fn element_adapter_mut_of(task: &mut Task) -> &mut dyn list::ElementAdapter<RunqueueTag, u32> {
        task as &mut dyn list::ElementAdapter<RunqueueTag, u32>
    }

    fn list_for_runqueue<'a>(&'a mut self, priority: u32) -> list::LinkedList<'a, Task, TaskList, RunqueueTag, { core::u32::MAX }> {
        list::LinkedList::new(&mut self.tasks, &mut self.runqueues[priority as usize], TaskPool::element_adapter_of, TaskPool::element_adapter_mut_of)
    }

    pub fn task_create(&mut self, tid: u32) -> Result<(), Error> {
        if self.get_task(tid).noarch().state != TaskState::Unused {
            return Err(Error::AlreadyExists);
        }
        let task = Task::create(tid)?;
        *self.get_task_mut(tid) = task;
        Ok(())
    } 

    fn enqueue_task<'t>(&mut self, task: &'t Task) {
        let mut list = self.list_for_runqueue(task.noarch().priority);
        list.push_back(task)
    }

    fn scheduler(&mut self, current_tid: u32) -> u32 {
        let current = self.get_task(current_tid);
        if current.noarch().task_type != TaskType::Idle && current.noarch().state == TaskState::Runnable {
            // The current task is still runnable. Enqueue into the runqueue.
            self.enqueue_task(current);
        }
        for priority in 0..TASK_PRIORITY_MAX {
            let mut list = self.list_for_runqueue(priority);
            if let Some(task) = list.pop_front() {
                return task.noarch().tid;
            }
        }
        0
    }

    pub fn task_switch(&mut self) {
        // stack_check();

        let prev_tid = self.current_tid;
        let next_tid = self.scheduler(prev_tid);
        self.get_task_mut(next_tid).noarch_mut().quantum = TASK_TIME_SLICE;
        if prev_tid == next_tid {
            // No runnable threads other than the current one. Continue executing
            // the current thread.
            return;
        }

        self.current_tid = next_tid;
        let prev = self.get_task(prev_tid);
        let next = self.get_task(next_tid);
        let (prev, next) = self.tasks.as_mut2(prev, next);
        Task::arch_task_switch(prev, next);

        // stack_check();
    }
}

struct ActiveIter<'a> {
    tid: u32,
    task_pool: &'a TaskPool,
}

impl<'a> Iterator for ActiveIter<'a> {
    type Item = &'a Task;

    fn next(&mut self) -> Option<Self::Item> {
        while self.tid < config::NUM_TASKS && self.task_pool.get_task(self.tid).noarch().state == TaskState::Unused {
            self.tid += 1;
        }
        if self.tid >= config::NUM_TASKS {
            None
        } else {
            Some(self.task_pool.get_task(self.tid))
        }
    }
}

pub struct NoarchTask {
    pub tid: u32,
    task_type: TaskType,
    pub state: TaskState,
    quantum: i32,
    priority: u32,
    runqueue_link: list::ListElement<u32>,
}

#[derive(PartialEq)]
pub enum TaskState {
    Unused = 0,
    Runnable,
    Blocked,
}

#[derive(PartialEq)]
pub enum TaskType {
    Idle = 0,
    User,
}

pub trait KArchTask {
    fn arch_task_create(task: NoarchTask, pc: usize) -> Result<Task, Error>;
    fn arch_task_switch(prev: &mut Task, next: &Task);
}

pub trait GetNoarchTask {
    fn noarch(&self) -> &NoarchTask;
    fn noarch_mut(&mut self) -> &mut NoarchTask;
}

pub trait TaskOps {
    fn create(tid: u32) -> Result<Task, Error>;
}

impl TaskOps for Task {
    fn create(tid: u32) -> Result<Task, Error> {
        Task::arch_task_create(
            NoarchTask {
                tid,
                task_type: TaskType::User,
                state: TaskState::Blocked,
                quantum: 0,
                priority: TASK_PRIORITY_MAX - 1,
                runqueue_link: list::ListElement::init(core::u32::MAX),
            },
            27,
        )
    }
}

pub fn get_task_pool() -> &'static mut TaskPool {
    unsafe { &mut TASK_POOL }
}

pub fn handle_timer_irq() {
    let task_pool = get_task_pool();

    let current = task_pool.current_mut().noarch_mut();
    current.quantum -= 1;
    if current.quantum < 0 {
        task_pool.task_switch();
    }
}
