pub use crate::arch::Task;
use crate::config;
use crate::error::Error;
use crate::list;
use crate::macros::*;
use crate::zeroed_array;

const TASK_PRIORITY_MAX: u32 = 8;
const TASK_TIME_SLICE: i32 = 10; // should meet timer intr cycle
pub const INIT_TASK_TID: u32 = 1;

type TaskList<'t> = [Task<'t>; config::NUM_TASKS as usize];
type RunQueue<'t> = list::ListLink<'t, Task<'t>>;

#[repr(align(16))]
pub struct TaskPool<'t> {
    tasks: TaskList<'t>,
    current: Option<&'t Task<'t>>,
    runqueues: [RunQueue<'t>; TASK_PRIORITY_MAX as usize],
}

static mut TASK_POOL: TaskPool<'static> = TaskPool {
    tasks: zeroed_array!(Task, config::NUM_TASKS as usize),
    current: None,
    runqueues: zeroed_array!(list::ListLink<'static, Task>, TASK_PRIORITY_MAX as usize),
};

trait TaskListOps<'t> {
    fn task(&self, tid: u32) -> &'t Task<'t>;
    fn task_mut(&mut self, tid: u32) -> &mut Task<'t>;
    fn as_mut1(&mut self, t: &'t Task<'t>) -> &'t mut Task<'t>;
    fn as_mut2(&mut self, t1: &'t Task<'t>, t2: &'t Task<'t>) -> (&mut Task<'t>, &mut Task<'t>);
    fn map_mut1<R, F: FnOnce(&mut Task<'t>) -> R>(&mut self, t1: &'t Task<'t>, f: F) -> R;
    fn map_mut2<R, F: FnOnce(&mut Task<'t>, &mut Task<'t>) -> R>(
        &mut self,
        t1: &'t Task<'t>,
        t2: &'t Task<'t>,
        f: F,
    ) -> R;
}

impl<'t> TaskListOps<'t> for TaskList<'t> {
    fn task(&self, tid: u32) -> &'t Task<'t> {
        unsafe { &*(self.get_unchecked(tid as usize) as *const Task) }
    }

    fn task_mut(&mut self, tid: u32) -> &mut Task<'t> {
        unsafe { self.get_unchecked_mut(tid as usize) }
    }

    fn as_mut1(&mut self, t: &'t Task<'t>) -> &'t mut Task<'t> {
        unsafe { &mut *(t as *const Task as *mut Task<'t>) }
    }

    fn as_mut2(&mut self, t1: &'t Task<'t>, t2: &'t Task<'t>) -> (&mut Task<'t>, &mut Task<'t>) {
        unsafe {
            (
                &mut *(t1 as *const Task as *mut Task),
                &mut *(t2 as *const Task as *mut Task),
            )
        }
    }

    fn map_mut1<R, F: FnOnce(&mut Task<'t>) -> R>(&mut self, t1: &'t Task<'t>, f: F) -> R {
        f(unsafe { &mut *(t1 as *const Task as *mut Task) })
    }

    fn map_mut2<R, F: FnOnce(&mut Task<'t>, &mut Task<'t>) -> R>(
        &mut self,
        t1: &'t Task<'t>,
        t2: &'t Task<'t>,
        f: F,
    ) -> R {
        f(unsafe { &mut *(t1 as *const Task as *mut Task) }, unsafe {
            &mut *(t2 as *const Task as *mut Task)
        })
    }
}

impl<'t> list::LinkAdapter<'t, Task<'t>, RunQueueTag> for Task<'t> {
    fn link(&'t self) -> &'t list::ListLink<'t, Task<'t>> {
        &self.noarch().runqueue_link
    }

    fn link_mut(&mut self) -> &mut list::ListLink<'t, Task<'t>> {
        &mut self.noarch_mut().runqueue_link
    }
}

impl<'t> list::ContainerAdapter<'t, Task<'t>> for TaskList<'t> {
    fn as_mut1(&mut self, t: &'t Task<'t>) -> &'t mut Task<'t> {
        <TaskList as TaskListOps>::as_mut1(self, t)
    }

    fn as_mut2(&mut self, t1: &'t Task<'t>, t2: &'t Task<'t>) -> (&mut Task<'t>, &mut Task<'t>) {
        <TaskList as TaskListOps>::as_mut2(self, t1, t2)
    }
}

struct RunQueueTag;

impl<'t> TaskPool<'t> {
    // fn active_iter(&self) -> ActiveIter {
    //     ActiveIter { tid: 0, task_pool: self }
    // }

    fn current(&self) -> &'t Task<'t> {
        unsafe { self.current.unwrap_unchecked() }
    }

    fn list_for_runqueue(
        &mut self,
        priority: u32,
    ) -> list::LinkedList<'_, 't, TaskList<'t>, Task<'t>, RunQueueTag> {
        list::LinkedList::new(&mut self.tasks, unsafe {
            self.runqueues.get_unchecked_mut(priority as usize)
        })
    }

    fn initiate_task(tid: u32, task: &mut Task<'t>, ip: usize) -> Result<(), Error> {
        if task.noarch().state != TaskState::Unused {
            return Err(Error::AlreadyExists);
        }
        *task = Task::create(tid, ip)?;
        Ok(())
    }

    pub fn create_user_task(&mut self, tid: u32, ip: usize) -> Result<(), Error> {
        self.tasks
            .map_mut1(self.tasks.task(tid), |task| {
                Self::initiate_task(tid, task, ip)
            })
            .map(|_| self.resume_task(self.tasks.task(tid)))
    }

    pub fn create_idle_task(&mut self) -> Result<(), Error> {
        self.tasks
            .map_mut1(self.tasks.task(0), |task| {
                Self::initiate_task(0, task, 0).map(|_| {
                    task.noarch_mut().task_type = TaskType::Idle;
                })
            })
            .map(|_| self.current = Some(self.tasks.task(0)))
    }

    fn enqueue_task(&mut self, task: &'t Task<'t>) {
        let mut list = self.list_for_runqueue(task.noarch().priority);
        list.push_back(task)
    }

    fn resume_task(&mut self, task: &'t Task<'t>) {
        self.tasks.map_mut1(task, |task| {
            task.noarch_mut().state = TaskState::Runnable;
        });
        self.enqueue_task(task);
    }

    fn scheduler(&mut self, current: &'t Task<'t>) -> &'t Task<'t> {
        if current.noarch().task_type != TaskType::Idle
            && current.noarch().state == TaskState::Runnable
        {
            // The current task is still runnable. Enqueue into the runqueue.
            self.enqueue_task(current);
        }
        for priority in 0..TASK_PRIORITY_MAX {
            let mut list = self.list_for_runqueue(priority);
            if let Some(task) = list.pop_front() {
                return task;
            }
        }
        self.tasks.task(0)
    }

    pub fn task_switch(&mut self) {
        // stack_check();

        let prev: &'t Task<'t> = unsafe { self.current.unwrap_unchecked() };
        let next: &'t Task<'t> = self.scheduler(prev);

        self.tasks.map_mut2(prev, next, |_prev, next| {
            next.noarch_mut().quantum = TASK_TIME_SLICE;
        });
        if prev.noarch().tid == next.noarch().tid {
            // No runnable threads other than the current one. Continue executing
            // the current thread.
            return;
        }

        self.current = Some(next);
        self.tasks
            .map_mut2(prev, next, |prev, next| Task::arch_task_switch(prev, next));

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
    fn create(tid: u32, ip: usize) -> Result<Task<'t>, Error>;
}

impl<'t> TaskOps<'t> for Task<'t> {
    fn create(tid: u32, ip: usize) -> Result<Task<'t>, Error> {
        Task::arch_task_create(
            NoarchTask {
                tid,
                task_type: TaskType::User,
                state: TaskState::Blocked,
                quantum: 0,
                priority: TASK_PRIORITY_MAX - 1,
                runqueue_link: list::ListLink::new(),
            },
            ip,
        )
    }
}

pub fn get_task_pool() -> &'static mut TaskPool<'static> {
    unsafe { &mut TASK_POOL }
}

pub fn handle_timer_irq() {
    let task_pool = get_task_pool();

    if let Some(current) = task_pool.current {
        let current = task_pool.tasks.as_mut1(current);
        current.noarch_mut().quantum -= 1;
        if current.noarch().quantum < 0 {
            task_pool.task_switch();
        }
    }
}
