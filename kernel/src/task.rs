pub use crate::arch::Task;
use crate::config;
use crate::ipc;
use core::cell::Cell;
use core::mem;
use klib::ipc::{Message, MessageType, Notifications};
use klib::list::{self, RemovableLinkedStackOps};
use klib::result::KResult;
use klib::zeroed_array;

const TASK_PRIORITY_MAX: u32 = 8;
const TASK_TIME_SLICE: i32 = 10; // should meet timer intr cycle
pub const KERNEL_TID: u32 = 0;
pub const INIT_TID: u32 = 1;

pub type TaskRef = &'static Task;
type TaskList = [Task; config::NUM_TASKS as usize];
type RunQueue = list::ListLink<'static, Task>;

#[repr(align(16))]
pub struct TaskPool {
    pub tasks: TaskList,
    runqueues: [RunQueue; TASK_PRIORITY_MAX as usize],
}

static mut TASK_POOL: TaskPool = TaskPool {
    tasks: zeroed_array!(Task, config::NUM_TASKS as usize),
    runqueues: zeroed_array!(list::ListLink<'static, Task>, TASK_PRIORITY_MAX as usize),
};

trait TaskListOps {
    fn task(&self, tid: u32) -> TaskRef;
}

impl TaskListOps for TaskList {
    fn task(&self, tid: u32) -> &'static Task {
        unsafe { &*(self.get_unchecked(tid as usize) as *const Task) }
    }
}

struct RunQueueTag;

impl list::LinkAdapter<'static, RunQueueTag> for Task {
    fn link(&self) -> &list::ListLink<'static, Task> {
        &self.noarch().runqueue_link
    }
    fn from_link<'a>(link: &'a list::ListLink<'static, Task>) -> &'a Task {
        unsafe {
            mem::transmute::<usize, &Task>(
                mem::transmute::<&list::ListLink<Task>, usize>(link)
                    & !(mem::size_of::<Task>() - 1),
            )
        }
    }
}

pub struct SendersTag;

impl list::LinkAdapter<'static, SendersTag> for Task {
    fn link(&self) -> &list::ListLink<'static, Task> {
        &self.noarch().sender_link
    }
    fn from_link<'a>(link: &'a list::ListLink<'static, Task>) -> &'a Task {
        unsafe {
            mem::transmute::<usize, &Task>(
                mem::transmute::<&list::ListLink<Task>, usize>(link)
                    & !(mem::size_of::<Task>() - 1),
            )
        }
    }
}

impl TaskPool {
    fn active_tasks(&self) -> ActiveTasks {
        ActiveTasks {
            tid: 0,
            task_pool: self,
        }
    }

    pub fn current(&self) -> TaskRef {
        Task::current()
    }

    fn list_for_runqueue(&self, priority: u32) -> list::LinkedList<'_, 'static, Task, RunQueueTag> {
        list::LinkedList::new(unsafe { self.runqueues.get_unchecked(priority as usize) })
    }

    fn initiate_task(tid: u32, task: TaskRef, pc: usize) -> KResult<()> {
        if task.noarch().state.get() != TaskState::Unused {
            return KResult::AlreadyExists;
        }
        Task::init(tid, task, pc)?;
        KResult::Ok(())
    }

    pub fn create_user_task(&self, tid: u32, pc: usize) -> KResult<()> {
        if tid > config::NUM_TASKS {
            return KResult::InvalidArg;
        }
        Self::initiate_task(tid, self.tasks.task(tid), pc)
            .map(|_| self.resume_task(self.tasks.task(tid)))
    }

    pub fn create_idle_task(&self) -> KResult<()> {
        Self::initiate_task(0, self.tasks.task(0), 0).map(|_| {
            let task = self.tasks.task(0);
            task.noarch().task_type.set(TaskType::Idle);
            Task::init_current(task);
        })
    }

    fn enqueue_task(&self, task: TaskRef) {
        let mut list = self.list_for_runqueue(task.priority());
        list.push_back(task);
    }

    // Suspends a task. Don't forget to update `task->src` as well!
    pub fn block_task(&self, task: TaskRef) {
        task.noarch().state.set(TaskState::Blocked);
    }

    pub fn resume_task(&self, task: TaskRef) {
        task.noarch().state.set(TaskState::Runnable);
        self.enqueue_task(task);
    }

    fn scheduler(&self, current: TaskRef) -> TaskRef {
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

    pub fn task_switch(&self) {
        // stack_check();

        let prev: TaskRef = self.current();
        let next: TaskRef = self.scheduler(prev);

        next.noarch().quantum.set(TASK_TIME_SLICE);
        if prev.tid() == next.tid() {
            // No runnable threads other than the current one. Continue executing
            // the current thread.
            return;
        }

        Task::arch_task_switch(prev, next);

        // stack_check();
    }

    pub fn set_current_timeout(&self, timeout: u32) -> KResult<()> {
        self.current().noarch().timeout.set(timeout);
        KResult::Ok(())
    }

    pub fn set_src_tid(&self, task: TaskRef, src_tid: u32) {
        task.noarch().src_tid.set(src_tid);
    }

    pub fn list_for_senders(
        &self,
        task: TaskRef,
    ) -> list::LinkedList<'_, 'static, Task, SendersTag> {
        list::LinkedList::new(&task.noarch().senders)
    }

    pub fn append_sender(&self, task: TaskRef, appended: TaskRef) {
        let mut list = self.list_for_senders(task);
        list.push_back(appended);
    }

    pub fn update_notifications<F: FnOnce(Notifications) -> Notifications>(
        &self,
        task: TaskRef,
        f: F,
    ) {
        task.noarch().notifications.update(|n| f(n));
    }

    pub fn lookup_task(&self, tid: u32) -> KResult<TaskRef> {
        if tid > config::NUM_TASKS {
            KResult::InvalidArg
        } else {
            let task = self.tasks.task(tid);
            if task.state() == TaskState::Unused {
                KResult::InvalidTask
            } else {
                KResult::Ok(task)
            }
        }
    }

    pub fn update_message<F: FnOnce(&mut Message)>(&self, task: TaskRef, f: F) {
        f(unsafe { &mut *task.noarch().message.as_ptr() })
    }
}

struct ActiveTasks<'a> {
    tid: u32,
    task_pool: &'a TaskPool,
}

impl<'a> Iterator for ActiveTasks<'a> {
    type Item = &'a Task;

    fn next(&mut self) -> Option<Self::Item> {
        while self.tid < config::NUM_TASKS
            && self.task_pool.tasks.task(self.tid).state() == TaskState::Unused
        {
            self.tid += 1;
        }
        if self.tid >= config::NUM_TASKS {
            None
        } else {
            let tid = self.tid;
            self.tid += 1;
            Some(self.task_pool.tasks.task(tid))
        }
    }
}

pub struct NoarchTask {
    tid: Cell<u32>,
    task_type: Cell<TaskType>,
    state: Cell<TaskState>,
    notifications: Cell<Notifications>,
    priority: Cell<u32>,
    quantum: Cell<i32>,
    message: Cell<Message>,
    src_tid: Cell<u32>,
    timeout: Cell<u32>,
    senders: list::ListLink<'static, Task>,
    runqueue_link: list::ListLink<'static, Task>,
    sender_link: list::ListLink<'static, Task>,
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

pub trait NotificationMessage {
    fn set_notification(&mut self, notifications: Notifications);
}

impl NotificationMessage for Message {
    fn set_notification(&mut self, notifications: Notifications) {
        self.message_type = MessageType::NOTIFICATIONS;
        self.src_tid = KERNEL_TID;
        self.raw.fill(0);
        self.set_payload(&notifications);
    }
}

pub trait KArchTask {
    fn arch_task_init(tid: u32, task: TaskRef, pc: usize) -> KResult<()>;
    fn arch_task_switch(prev: &Task, next: &Task);
    fn init_current(task: TaskRef);
    fn current() -> TaskRef;
}

pub trait GetNoarchTask {
    fn noarch(&self) -> &NoarchTask;
}

pub trait TaskOps {
    fn init(tid: u32, task: TaskRef, pc: usize) -> KResult<()>;
    fn tid(&self) -> u32;
    fn priority(&self) -> u32;
    fn quantum(&self) -> i32;
    fn timeout(&self) -> u32;
    fn src_tid(&self) -> u32;
    fn task_type(&self) -> TaskType;
    fn state(&self) -> TaskState;
    fn notifications(&self) -> Notifications;
}

impl TaskOps for Task {
    fn init(tid: u32, task: TaskRef, pc: usize) -> KResult<()> {
        task.noarch().tid.set(tid);
        task.noarch().task_type.set(TaskType::User);
        task.noarch().state.set(TaskState::Blocked);
        task.noarch().priority.set(TASK_PRIORITY_MAX - 1);
        task.noarch().quantum.set(0);
        task.noarch().message.set(unsafe { mem::zeroed() });
        task.noarch().notifications.set(Notifications::none());
        task.noarch().src_tid.set(0);
        task.noarch().timeout.set(0);
        task.noarch().senders.reset();
        task.noarch().runqueue_link.reset();
        task.noarch().sender_link.reset();
        Task::arch_task_init(tid, task, pc)
    }

    fn tid(&self) -> u32 {
        self.noarch().tid.get()
    }
    fn priority(&self) -> u32 {
        self.noarch().priority.get()
    }
    fn quantum(&self) -> i32 {
        self.noarch().quantum.get()
    }
    fn timeout(&self) -> u32 {
        self.noarch().timeout.get()
    }
    fn src_tid(&self) -> u32 {
        self.noarch().src_tid.get()
    }
    fn task_type(&self) -> TaskType {
        self.noarch().task_type.get()
    }
    fn state(&self) -> TaskState {
        self.noarch().state.get()
    }
    fn notifications(&self) -> Notifications {
        self.noarch().notifications.get()
    }
}

pub fn get_task_pool() -> &'static TaskPool {
    unsafe { &TASK_POOL }
}

pub fn handle_timer_irq() {
    let task_pool = get_task_pool();

    let resumed_by_timeout = task_pool
        .active_tasks()
        .filter(|task: &&Task| task.timeout() > 0)
        .filter(|task| {
            let next_timeout = task.timeout() - 1;
            task.noarch().timeout.set(next_timeout);
            next_timeout == 0
        })
        .map(|task| ipc::notify(task_pool, task, Notifications::timer()))
        .count()
        > 0;

    let current = task_pool.current();
    current.noarch().quantum.update(|quantum| quantum - 1);
    if current.quantum() < 0 || resumed_by_timeout {
        task_pool.task_switch();
    }
}
