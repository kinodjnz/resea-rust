pub use crate::arch::Task;
use crate::config;
use crate::list;
use crate::macros::*;
use crate::mmio;
use crate::result::KResult;
use crate::zeroed_array;
use core::cell::{Cell, UnsafeCell};
use core::mem;
use core::ops::BitOr;

const TASK_PRIORITY_MAX: u32 = 8;
const TASK_TIME_SLICE: i32 = 10; // should meet timer intr cycle
pub const KERNEL_TID: u32 = 0;
pub const INIT_TID: u32 = 1;

type TaskCell = UnsafeCell<Task>;
pub type TaskRef = &'static TaskCell;
type TaskList = [UnsafeCell<Task>; config::NUM_TASKS as usize];
type RunQueue = list::ListLink<Task>;

#[repr(align(16))]
pub struct TaskPool {
    pub tasks: TaskList,
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
    // fn task_mut(&mut self, tid: u32) -> &mut Task;
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

    // fn task_mut(&mut self, tid: u32) -> &mut Task {
    //     unsafe { self.get_unchecked_mut(tid as usize).get_mut() }
    // }

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

struct RunQueueTag;

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

pub struct SendersTag;

impl list::LinkAdapter<Task, SendersTag> for Task {
    fn link(&self) -> &list::ListLink<Task> {
        &self.noarch().sender_link
    }

    fn link_mut(&mut self) -> &mut list::ListLink<Task> {
        &mut self.noarch_mut().sender_link
    }
}

impl TaskPool {
    // fn active_iter(&self) -> ActiveIter {
    //     ActiveIter { tid: 0, task_pool: self }
    // }

    pub fn current(&self) -> TaskRef {
        unsafe { self.current.unwrap_unchecked() }
    }

    fn list_for_runqueue(
        &mut self,
        priority: u32,
    ) -> list::LinkedList<'_, TaskList, Task, RunQueueTag> {
        list::LinkedList::new(
            &self.tasks,
            unsafe { self.runqueues.get_unchecked_mut(priority as usize) }.get_mut(),
        )
    }

    fn initiate_task(tid: u32, task: &mut Task, ip: usize) -> KResult<()> {
        if task.noarch().state.get() != TaskState::Unused {
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
        list.push_back(task);
    }

    // Suspends a task. Don't forget to update `task->src` as well!
    pub fn block_task(&mut self, task: TaskRef) {
        task.noarch().state.set(TaskState::Blocked);
    }

    pub fn resume_task(&mut self, task: TaskRef) {
        task.noarch().state.set(TaskState::Runnable);
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

        next.noarch().quantum.set(TASK_TIME_SLICE);
        if prev.tid() == next.tid() {
            // No runnable threads other than the current one. Continue executing
            // the current thread.
            return;
        }

        self.current = Some(next);
        Task::arch_task_switch(unsafe { &*prev.get() }, unsafe { &*next.get() });

        // stack_check();
    }

    pub fn set_current_timeout(&mut self, timeout: u32) -> KResult<()> {
        if let Some(current) = self.current {
            current.noarch().timeout.set(timeout);
            KResult::Ok(())
        } else {
            KResult::NotReady
        }
    }

    pub fn set_src_tid(&mut self, task: TaskRef, src_tid: u32) {
        task.noarch().src_tid.set(src_tid);
    }

    pub fn list_for_senders(
        &mut self,
        task: TaskRef,
    ) -> list::LinkedList<'_, TaskList, Task, SendersTag> {
        list::LinkedList::new(&self.tasks, unsafe { &mut *task.noarch().senders.get() })
    }

    pub fn append_sender(&mut self, task: TaskRef, appended: TaskRef) {
        let mut list = self.list_for_senders(task);
        list.push_back(appended);
    }

    pub fn update_notifications<F: FnOnce(Notifications) -> Notifications>(
        &mut self,
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

    pub fn update_message<F: FnOnce(&mut Message)>(&mut self, task: TaskRef, f: F) {
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
    pub tid: u32,
    task_type: TaskType,
    state: Cell<TaskState>,
    priority: u32,
    quantum: Cell<i32>,
    message: Cell<Message>,
    src_tid: Cell<u32>,
    notifications: Cell<Notifications>,
    timeout: Cell<u32>,
    senders: UnsafeCell<list::ListLink<Task>>,
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
    message_type: MessageType,
    src_tid: u32,
    raw: [u8; 24],
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
    fn arch_task_create(task: NoarchTask, pc: usize) -> KResult<Task>;
    fn arch_task_switch(prev: &Task, next: &Task);
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
                state: TaskState::Blocked.into(),
                priority: TASK_PRIORITY_MAX - 1,
                quantum: 0.into(),
                message: unsafe { core::mem::zeroed() },
                notifications: Notifications::none().into(),
                src_tid: 0.into(),
                timeout: 0.into(),
                senders: list::ListLink::new().into(),
                runqueue_link: list::ListLink::new(),
                sender_link: list::ListLink::new(),
            },
            ip,
        )
    }
}

pub trait TaskCellOps {
    fn tid(&self) -> u32;
    fn priority(&self) -> u32;
    fn quantum(&self) -> i32;
    fn src_tid(&self) -> u32;
    fn task_type(&self) -> TaskType;
    fn state(&self) -> TaskState;
    fn notifications(&self) -> Notifications;
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
        self.noarch().quantum.get()
    }
    fn src_tid(&self) -> u32 {
        self.noarch().src_tid.get()
    }
    fn task_type(&self) -> TaskType {
        self.noarch().task_type
    }
    fn state(&self) -> TaskState {
        self.noarch().state.get()
    }
    fn notifications(&self) -> Notifications {
        self.noarch().notifications.get()
    }
}

pub fn get_task_pool() -> &'static mut TaskPool {
    unsafe { &mut TASK_POOL }
}

pub fn handle_timer_irq() {
    let task_pool = get_task_pool();

    if let Some(current) = task_pool.current {
        current.noarch().quantum.update(|quantum| quantum - 1);
        if current.quantum() < 0 {
            task_pool.task_switch();
        }
    }
}
