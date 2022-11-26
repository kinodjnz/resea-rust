pub use crate::arch::Task;
use crate::config;
use crate::list;
use crate::macros::*;
use crate::result::KResult;
use crate::zeroed_array;
use core::cell::UnsafeCell;

const TASK_PRIORITY_MAX: u32 = 8;
const TASK_TIME_SLICE: i32 = 10; // should meet timer intr cycle
pub const INIT_TASK_TID: u32 = 1;

type TaskCell = UnsafeCell<Task>;
type TaskRef = &'static TaskCell;
type TaskList = [UnsafeCell<Task>; config::NUM_TASKS as usize];
type RunQueue = list::ListLink<Task>;

#[repr(align(16))]
pub struct TaskPool {
    tasks: TaskList,
    current: Option<&'static UnsafeCell<Task>>,
    runqueues: [UnsafeCell<RunQueue>; TASK_PRIORITY_MAX as usize],
}

static mut TASK_POOL: TaskPool = TaskPool {
    tasks: zeroed_array!(UnsafeCell<Task>, config::NUM_TASKS as usize),
    current: None,
    runqueues: zeroed_array!(UnsafeCell<list::ListLink<Task>>, TASK_PRIORITY_MAX as usize),
};

trait TaskListOps {
    fn task(&self, tid: u32) -> TaskRef;
    fn task_mut(&mut self, tid: u32) -> &mut Task;
    fn map_mut1<R, F: FnOnce(&mut Task) -> R>(&self, t1: TaskRef, f: F) -> R;
    fn map_mut2<R, F: FnOnce(&mut Task, &mut Task) -> R>(
        &self,
        t1: TaskRef,
        t2: TaskRef,
        f: F,
    ) -> R;
}

impl TaskListOps for TaskList {
    fn task(&self, tid: u32) -> &'static UnsafeCell<Task> {
        unsafe { &*(self.get_unchecked(tid as usize) as *const UnsafeCell<Task>) }
    }

    fn task_mut(&mut self, tid: u32) -> &mut Task {
        unsafe { self.get_unchecked_mut(tid as usize).get_mut() }
    }

    fn map_mut1<R, F: FnOnce(&mut Task) -> R>(&self, t1: TaskRef, f: F) -> R {
        f(unsafe { &mut *t1.get() })
    }

    fn map_mut2<R, F: FnOnce(&mut Task, &mut Task) -> R>(
        &self,
        t1: TaskRef,
        t2: TaskRef,
        f: F,
    ) -> R {
        if t1.get() == t2.get() {
            kpanic!(b"Mutated tasks are identical\n");
        }
        f(unsafe { &mut *t1.get() }, unsafe { &mut *t2.get() })
    }
}

impl list::LinkAdapter<Task, RunQueueTag> for Task {
    fn link(&self) -> &list::ListLink<Task> {
        &self.noarch().runqueue_link
    }

    fn link_mut(&mut self) -> &mut list::ListLink<Task> {
        &mut self.noarch_mut().runqueue_link
    }
}

impl list::ContainerAdapter<Task> for TaskList {
    fn map_mut1<R, F: FnOnce(&mut Task) -> R>(&self, t1: TaskRef, f: F) -> R {
        <TaskList as TaskListOps>::map_mut1(self, t1, f)
    }

    fn map_mut2<R, F: FnOnce(&mut Task, &mut Task) -> R>(
        &self,
        t1: TaskRef,
        t2: TaskRef,
        f: F,
    ) -> R {
        <TaskList as TaskListOps>::map_mut2(self, t1, t2, f)
    }
}

struct RunQueueTag;

impl TaskPool {
    // fn active_iter(&self) -> ActiveIter {
    //     ActiveIter { tid: 0, task_pool: self }
    // }

    fn current(&self) -> TaskRef {
        unsafe { self.current.unwrap_unchecked() }
    }

    fn list_for_runqueue(
        &mut self,
        priority: u32,
    ) -> list::LinkedList<'_, TaskList, Task, RunQueueTag> {
        list::LinkedList::new(&self.tasks, unsafe {
            self.runqueues.get_unchecked_mut(priority as usize)
        })
    }

    fn initiate_task(tid: u32, task: &mut Task, ip: usize) -> KResult<()> {
        if task.noarch().state != TaskState::Unused {
            return KResult::AlreadyExists;
        }
        *task = Task::create(tid, ip)?;
        KResult::Ok(())
    }

    pub fn create_user_task(&mut self, tid: u32, ip: usize) -> KResult<()> {
        self.tasks
            .map_mut1(self.tasks.task(tid), |task| {
                Self::initiate_task(tid, task, ip)
            })
            .map(|_| self.resume_task(self.tasks.task(tid)))
    }

    pub fn create_idle_task(&mut self) -> KResult<()> {
        self.tasks
            .map_mut1(self.tasks.task(0), |task| {
                Self::initiate_task(0, task, 0).map(|_| {
                    task.noarch_mut().task_type = TaskType::Idle;
                })
            })
            .map(|_| self.current = Some(self.tasks.task(0)))
    }

    fn enqueue_task(&mut self, task: TaskRef) {
        let mut list = self.list_for_runqueue(task.priority());
        list.push_back(task)
    }

    fn resume_task(&mut self, task: TaskRef) {
        self.tasks.map_mut1(task, |task| {
            task.noarch_mut().state = TaskState::Runnable;
        });
        self.enqueue_task(task);
    }

    fn scheduler(&mut self, current: TaskRef) -> TaskRef {
        if current.task_type() != TaskType::Idle && current.state() == TaskState::Runnable {
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

        let prev: TaskRef = self.current();
        let next: TaskRef = self.scheduler(prev);

        self.tasks
            .map_mut1(next, |next| next.noarch_mut().quantum = TASK_TIME_SLICE);
        if prev.tid() == next.tid() {
            // No runnable threads other than the current one. Continue executing
            // the current thread.
            return;
        }

        self.current = Some(next);
        self.tasks
            .map_mut2(prev, next, |prev, next| Task::arch_task_switch(prev, next));

        // stack_check();
    }

    pub fn set_current_timeout(&mut self, timeout: u32) -> KResult<()> {
        if let Some(current) = self.current {
            self.tasks
                .map_mut1(current, |current| current.noarch_mut().timeout = timeout);
            KResult::Ok(())
        } else {
            KResult::NotReady
        }
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

pub struct NoarchTask {
    pub tid: u32,
    task_type: TaskType,
    pub state: TaskState,
    quantum: i32,
    priority: u32,
    timeout: u32,
    runqueue_link: list::ListLink<Task>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum TaskState {
    Unused = 0,
    Runnable,
    Blocked,
}

#[derive(PartialEq, Clone, Copy)]
pub enum TaskType {
    Idle = 0,
    User,
}

pub trait KArchTask {
    fn arch_task_create(task: NoarchTask, pc: usize) -> KResult<Task>;
    fn arch_task_switch(prev: &mut Task, next: &mut Task);
}

pub trait GetNoarchTask {
    fn noarch(&self) -> &NoarchTask;
    fn noarch_mut(&mut self) -> &mut NoarchTask;
}

pub trait TaskOps {
    fn create(tid: u32, ip: usize) -> KResult<Task>;
}

impl TaskOps for Task {
    fn create(tid: u32, ip: usize) -> KResult<Task> {
        Task::arch_task_create(
            NoarchTask {
                tid,
                task_type: TaskType::User,
                state: TaskState::Blocked,
                quantum: 0,
                priority: TASK_PRIORITY_MAX - 1,
                timeout: 0,
                runqueue_link: list::ListLink::new(),
            },
            ip,
        )
    }
}

pub trait TaskCellOps {
    fn tid(&self) -> u32;
    fn priority(&self) -> u32;
    fn quantum(&self) -> i32;
    fn task_type(&self) -> TaskType;
    fn state(&self) -> TaskState;
    fn noarch(&self) -> &NoarchTask;
}

impl TaskCellOps for TaskCell {
    fn noarch(&self) -> &NoarchTask {
        unsafe { (&*self.get()).noarch() }
    }
    fn tid(&self) -> u32 {
        self.noarch().tid
    }
    fn priority(&self) -> u32 {
        self.noarch().priority
    }
    fn quantum(&self) -> i32 {
        self.noarch().quantum
    }
    fn task_type(&self) -> TaskType {
        self.noarch().task_type
    }
    fn state(&self) -> TaskState {
        self.noarch().state
    }
}

pub fn get_task_pool() -> &'static mut TaskPool {
    unsafe { &mut TASK_POOL }
}

pub fn handle_timer_irq() {
    let task_pool = get_task_pool();

    if let Some(current) = task_pool.current {
        task_pool
            .tasks
            .map_mut1(current, |current| current.noarch_mut().quantum -= 1);
        if current.quantum() < 0 {
            task_pool.task_switch();
        }
    }
}
