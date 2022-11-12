pub use crate::arch::Task;
use crate::config;
use crate::error::Error;
use crate::macros::*;
use crate::zeroed_array;
use crate::list;

const TASK_PRIORITY_MAX: u32 = 8;
const TASK_TIME_SLICE: i32 = 10; // should meet timer intr cycle

type TaskList<'t> = [Task<'t>; config::NUM_TASKS as usize];

#[repr(align(16))]
pub struct TaskPool<'t> {
    tasks: TaskList<'t>,
    idle_task: Task<'t>,
    current: Option<&'t Task<'t>>,
    runqueues: [list::ListLink<'t, Task<'t>>; TASK_PRIORITY_MAX as usize],
}

static mut TASK_POOL: TaskPool<'static> = TaskPool {
    tasks: zeroed_array!(Task, config::NUM_TASKS as usize),
    idle_task: zeroed_const!(Task),
    current: None,
    runqueues: zeroed_array!(list::ListLink<'static, Task>, TASK_PRIORITY_MAX as usize),
};

trait TaskListOps<'t> {
    fn task(&self, tid: u32) -> &'t Task<'t>;
    fn task_mut(&mut self, tid: u32) -> &mut Task<'t>;
    fn as_mut(&mut self, t: &'t Task<'t>) -> &'t mut Task<'t>;
    fn as_mut2(&mut self, t1: &'t Task<'t>, t2: &'t Task<'t>) -> (&mut Task<'t>, &mut Task<'t>);
}

impl<'t> TaskListOps<'t> for TaskList<'t> {
    fn task(&self, tid: u32) -> &'t Task<'t> {
        unsafe {
            &*(self.get_unchecked(tid as usize) as *const Task)
        }
    }

    fn task_mut(&mut self, tid: u32) -> &mut Task<'t> {
        unsafe {
            self.get_unchecked_mut(tid as usize)
        }
    }

    fn as_mut(&mut self, t: &'t Task<'t>) -> &'t mut Task<'t> {
        unsafe {
            &mut *(t as *const Task as *mut Task<'t>)
        }
    }

    fn as_mut2(&mut self, t1: &'t Task<'t>, t2: &'t Task<'t>) -> (&mut Task<'t>, &mut Task<'t>) {
        unsafe {
            (&mut *(t1 as *const Task as *mut Task), &mut *(t2 as *const Task as *mut Task))
        }
    }
}

impl<'t> list::LinkAdapter<'t, Task<'t>, RunqueueTag> for Task<'t> {
    fn link(&'t self) -> &'t list::ListLink<'t, Task<'t>> {
        &self.noarch().runqueue_link
    }

    fn link_mut(&mut self) -> &mut list::ListLink<'t, Task<'t>> {
        &mut self.noarch_mut().runqueue_link
    }
}

impl<'t> list::ContainerAdapter<'t, Task<'t>> for TaskList<'t> {
    fn as_mut(&mut self, t: &'t Task<'t>) -> &'t mut Task<'t> {
        <TaskList as TaskListOps>::as_mut(self, t)
    }

    fn as_mut2(&mut self, t1: &'t Task<'t>, t2: &'t Task<'t>) -> (&mut Task<'t>, &mut Task<'t>) {
        <TaskList as TaskListOps>::as_mut2(self, t1, t2)
    }
}

struct RunqueueTag;

impl<'t> TaskPool<'t> {
    fn get_task(&self, tid: u32) -> &'t Task<'t> {
        self.tasks.task(tid)
    }

    fn get_task_mut(&mut self, tid: u32) -> &mut Task<'t> {
        self.tasks.task_mut(tid)
    }

    fn as_mut(&mut self, t: &'t Task<'t>) -> &mut Task<'t> {
        unsafe {
            &mut *(t as *const Task as *mut Task)
        }
    }

    // fn active_iter(&self) -> ActiveIter {
    //     ActiveIter { tid: 0, task_pool: self }
    // }

    fn current(&self) -> &'t Task<'t> {
        unsafe {
            self.current.unwrap_unchecked()
        }
    }

    fn list_for_runqueue<'a>(&'a mut self, priority: u32) -> list::LinkedList<'a, 't, TaskList<'t>, Task<'t>, RunqueueTag> {
        list::LinkedList::new(&mut self.tasks, unsafe { self.runqueues.get_unchecked_mut(priority as usize) })
    }

    pub fn task_create(&mut self, tid: u32) -> Result<(), Error> {
        if self.get_task(tid).noarch().state != TaskState::Unused {
            return Err(Error::AlreadyExists);
        }
        let task = Task::create(tid)?;
        *self.get_task_mut(tid) = task;
        Ok(())
    } 

    fn enqueue_task(&mut self, task: &'t Task<'t>) {
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

        let prev_tid = self.current().noarch().tid;
        let next_tid = self.scheduler(prev_tid);
        self.get_task_mut(next_tid).noarch_mut().quantum = TASK_TIME_SLICE;
        if prev_tid == next_tid {
            // No runnable threads other than the current one. Continue executing
            // the current thread.
            return;
        }

        let prev = self.get_task(prev_tid);
        let next = self.get_task(next_tid);
        self.current = Some(next);
        let (prev, next) = self.tasks.as_mut2(prev, next);
        Task::arch_task_switch(prev, next);

        // stack_check();
    }
}

// struct ActiveIter<'a> {
//     tid: u32,
//     task_pool: &'a TaskPool<'t>,
// }

// impl<'a> Iterator for ActiveIter<'a> {
//     type Item = &'a Task;

//     fn next(&mut self) -> Option<Self::Item> {
//         while self.tid < config::NUM_TASKS && self.task_pool.get_task(self.tid).noarch().state == TaskState::Unused {
//             self.tid += 1;
//         }
//         if self.tid >= config::NUM_TASKS {
//             None
//         } else {
//             Some(self.task_pool.get_task(self.tid))
//         }
//     }
// }

pub struct NoarchTask<'t> {
    pub tid: u32,
    task_type: TaskType,
    pub state: TaskState,
    quantum: i32,
    priority: u32,
    runqueue_link: list::ListLink<'t, Task<'t>>,
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

pub trait KArchTask<'t> {
    fn arch_task_create(task: NoarchTask<'t>, pc: usize) -> Result<Task<'t>, Error>;
    fn arch_task_switch(prev: &mut Task<'t>, next: &mut Task<'t>);
}

pub trait GetNoarchTask<'t> {
    fn noarch(&'t self) -> &'t NoarchTask<'t>;
    fn noarch_mut(&mut self) -> &mut NoarchTask<'t>;
}

pub trait TaskOps<'t> {
    fn create(tid: u32) -> Result<Task<'t>, Error>;
}

impl<'t> TaskOps<'t> for Task<'t> {
    fn create(tid: u32) -> Result<Task<'t>, Error> {
        Task::arch_task_create(
            NoarchTask {
                tid,
                task_type: TaskType::User,
                state: TaskState::Blocked,
                quantum: 0,
                priority: TASK_PRIORITY_MAX - 1,
                runqueue_link: list::ListLink::new(),
            },
            27,
        )
    }
}

pub fn get_task_pool() -> &'static mut TaskPool<'static> {
    unsafe { &mut TASK_POOL }
}

pub fn handle_timer_irq() {
    let task_pool = get_task_pool();

    let current = task_pool.as_mut(task_pool.current());
    current.noarch_mut().quantum -= 1;
    if current.noarch().quantum < 0 {
        task_pool.task_switch();
    }
}
