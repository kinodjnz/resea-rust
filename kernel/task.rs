pub use crate::arch::Task;
use crate::config;
use crate::list;
use crate::mmio;
use crate::result::KResult;
use crate::zeroed_array;
use core::cell::Cell;
use core::mem;
use core::ops::BitOr;

const TASK_PRIORITY_MAX: u32 = 8;
const TASK_TIME_SLICE: i32 = 10; // should meet timer intr cycle
pub const KERNEL_TID: u32 = 0;
pub const INIT_TID: u32 = 1;

pub type TaskRef = &'static Task;
type TaskList = [Task; config::NUM_TASKS as usize];
type RunQueue = list::ListLink<Task>;

#[repr(align(16))]
pub struct TaskPool {
    pub tasks: TaskList,
    current: Cell<Option<&'static Task>>,
    runqueues: [RunQueue; TASK_PRIORITY_MAX as usize],
}

static mut TASK_POOL: TaskPool = TaskPool {
    tasks: zeroed_array!(Task, config::NUM_TASKS as usize),
    current: zeroed_const!(Cell<Option<&'static Task>>),
    runqueues: zeroed_array!(list::ListLink<Task>, TASK_PRIORITY_MAX as usize),
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

impl list::LinkAdapter<Task, RunQueueTag> for Task {
    fn link(&self) -> &list::ListLink<Task> {
        &self.noarch().runqueue_link
    }
}

pub struct SendersTag;

impl list::LinkAdapter<Task, SendersTag> for Task {
    fn link(&self) -> &list::ListLink<Task> {
        &self.noarch().sender_link
    }
}

impl TaskPool {
    // fn active_iter(&self) -> ActiveIter {
    //     ActiveIter { tid: 0, task_pool: self }
    // }

    pub fn current(&self) -> TaskRef {
        unsafe { self.current.get().unwrap_unchecked() }
    }

    fn list_for_runqueue(&self, priority: u32) -> list::LinkedList<'_, Task, RunQueueTag> {
        list::LinkedList::new(unsafe { self.runqueues.get_unchecked(priority as usize) })
    }

    fn initiate_task(tid: u32, task: TaskRef, ip: usize) -> KResult<()> {
        if task.noarch().state.get() != TaskState::Unused {
            return KResult::AlreadyExists;
        }
        Task::init(tid, task, ip)?;
        KResult::Ok(())
    }

    pub fn create_user_task(&self, tid: u32, ip: usize) -> KResult<()> {
        Self::initiate_task(tid, self.tasks.task(tid), ip)
            .map(|_| self.resume_task(self.tasks.task(tid)))
    }

    pub fn create_idle_task(&self) -> KResult<()> {
        Self::initiate_task(0, self.tasks.task(0), 0).map(|_| {
            let task = self.tasks.task(0);
            task.noarch().task_type.set(TaskType::Idle);
            self.current.set(Some(task));
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

        self.current.set(Some(next));
        Task::arch_task_switch(prev, next);

        // stack_check();
    }

    pub fn set_current_timeout(&self, timeout: u32) -> KResult<()> {
        if let Some(current) = self.current.get() {
            current.noarch().timeout.set(timeout);
            KResult::Ok(())
        } else {
            KResult::NotReady
        }
    }

    pub fn set_src_tid(&self, task: TaskRef, src_tid: u32) {
        task.noarch().src_tid.set(src_tid);
    }

    pub fn list_for_senders(&self, task: TaskRef) -> list::LinkedList<'_, Task, SendersTag> {
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
    tid: Cell<u32>,
    task_type: Cell<TaskType>,
    state: Cell<TaskState>,
    priority: Cell<u32>,
    quantum: Cell<i32>,
    message: Cell<Message>,
    src_tid: Cell<u32>,
    notifications: Cell<Notifications>,
    timeout: Cell<u32>,
    senders: list::ListLink<Task>,
    runqueue_link: list::ListLink<Task>,
    sender_link: list::ListLink<Task>,
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

#[derive(Clone, Copy)]
pub struct Notifications(u8);

#[allow(unused)]
impl Notifications {
    const TIMER: u8 = 1 << 0;
    const IRQ: u8 = 1 << 1;
    const ABORTED: u8 = 1 << 2;
    const ASYNC: u8 = 1 << 3;

    pub fn from_u32(n: u32) -> Notifications {
        Notifications(n as u8)
    }

    pub fn aborted() -> Notifications {
        Notifications(Self::ABORTED)
    }
    pub fn clear(&self, notifications: Notifications) -> Notifications {
        Notifications(self.0 & !notifications.0)
    }
    pub fn none() -> Notifications {
        Notifications(0)
    }
    pub fn is_aborted(&self) -> bool {
        self.0 & Self::ABORTED != 0
    }
    pub fn exists(&self) -> bool {
        self.0 != 0
    }
}

impl BitOr for Notifications {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

pub struct MessageType(pub u32);

impl MessageType {
    const NOTIFICATIONS: MessageType = MessageType(1);
}

pub struct Message {
    pub message_type: MessageType,
    pub src_tid: u32,
    pub raw: [u8; 24],
}

impl Message {
    pub fn set_notification(&mut self, notifications: Notifications) {
        self.message_type = MessageType::NOTIFICATIONS;
        self.src_tid = KERNEL_TID;
        mmio::mzero_array(&mut self.raw);
        mmio::memcpy_align4(
            self.raw.as_mut_ptr() as *mut Notifications,
            &notifications,
            1,
        );
    }
}

pub trait KArchTask {
    fn arch_task_init(tid: u32, task: TaskRef, pc: usize) -> KResult<()>;
    fn arch_task_switch(prev: &Task, next: &Task);
}

pub trait GetNoarchTask {
    fn noarch(&self) -> &NoarchTask;
}

pub trait TaskOps {
    fn init(tid: u32, task: TaskRef, ip: usize) -> KResult<()>;
    fn tid(&self) -> u32;
    fn priority(&self) -> u32;
    fn quantum(&self) -> i32;
    fn src_tid(&self) -> u32;
    fn task_type(&self) -> TaskType;
    fn state(&self) -> TaskState;
    fn notifications(&self) -> Notifications;
}

impl TaskOps for Task {
    fn init(tid: u32, task: TaskRef, ip: usize) -> KResult<()> {
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
        Task::arch_task_init(tid, task, ip)
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

    if let Some(current) = task_pool.current.get() {
        current.noarch().quantum.update(|quantum| quantum - 1);
        if current.quantum() < 0 {
            task_pool.task_switch();
        }
    }
}
