use crate::mmio;
use crate::result::KResult;
use crate::task::{Message, Notifications, TaskOps, TaskPool, TaskRef, TaskState};
use core::u32;

#[derive(Clone, Copy)]
pub struct IpcFlags(u8);

#[allow(unused)]
impl IpcFlags {
    const SEND: u8 = 1 << 0;
    const RECV: u8 = 1 << 1;
    const NOBLOCK: u8 = 1 << 2;
    const KERNEL: u8 = 1 << 3; // Internally used by kernel.

    pub fn from_u32(flags: u32) -> IpcFlags {
        IpcFlags(flags as u8)
    }

    pub fn send() -> IpcFlags {
        IpcFlags(Self::SEND)
    }
    pub fn recv() -> IpcFlags {
        IpcFlags(Self::RECV)
    }
    pub fn call() -> IpcFlags {
        IpcFlags(Self::SEND | Self::RECV)
    }
    pub fn is_send(&self) -> bool {
        self.0 & Self::SEND != 0
    }
    pub fn is_recv(&self) -> bool {
        self.0 & Self::RECV != 0
    }
    pub fn is_noblock(&self) -> bool {
        self.0 & Self::NOBLOCK != 0
    }
    pub fn is_kernel(&self) -> bool {
        self.0 & Self::KERNEL != 0
    }
}

pub struct IpcSrcTask;

impl IpcSrcTask {
    const ANY: u32 = 0;
    const DENY: u32 = u32::MAX;
}

pub fn call(
    task_pool: &mut TaskPool,
    dst_task: TaskRef,
    src_tid: u32,
    message: &mut Message,
    flags: IpcFlags,
) -> KResult<()> {
    send(task_pool, dst_task, message, flags).and_then(|_| recv(task_pool, src_tid, message, flags))
}

pub fn send(
    task_pool: &mut TaskPool,
    dst_task: TaskRef,
    message: &mut Message,
    flags: IpcFlags,
) -> KResult<()> {
    let receiver_is_ready = dst_task.state() == TaskState::Blocked
        && (dst_task.src_tid() == IpcSrcTask::ANY
            || dst_task.src_tid() == task_pool.current().tid());
    if !receiver_is_ready {
        if flags.is_noblock() {
            return KResult::WouldBlock;
        }

        let current = task_pool.current();
        task_pool.set_src_tid(current, IpcSrcTask::DENY);
        task_pool.block_task(current);
        task_pool.append_sender(dst_task, current);
        task_pool.task_switch();

        let current = task_pool.current();
        if current.notifications().is_aborted() {
            task_pool.update_notifications(current, |n| n.clear(Notifications::aborted()));
            return KResult::Aborted;
        }
    }
    task_pool.update_message(dst_task, |dst_msg| mmio::memcpy_align4(dst_msg, message, 1));
    task_pool.resume_task(dst_task);

    KResult::Ok(())
}

/// Resumes a sender task for the `receiver` tasks and updates `receiver->src`
/// properly.
fn resume_sender(task_pool: &mut TaskPool, receiver: TaskRef, src_tid: u32) {
    if let Some(sender) = task_pool
        .list_for_senders(receiver)
        .iter()
        .find(|sender| src_tid == IpcSrcTask::ANY || src_tid == sender.tid())
    {
        // DEBUG_ASSERT(sender->state == TASK_BLOCKED);
        // DEBUG_ASSERT(sender->src == IPC_DENY);
        task_pool.resume_task(sender);
        task_pool.list_for_senders(receiver).remove(sender);
        // If src == IPC_ANY, allow only `sender` to send a message. Let's
        // consider the following situation to understand why:
        //
        //     [Sender A]              [Receiver C]              [Sender B]
        //         .                        |                        |
        // in C's sender queue              |                        |
        //         .                        |                        |
        //         .        Resume          |                        |
        //         + <--------------------- |                        |
        //         .                        .    Try sending (X)     |
        //         .                        + <--------------------- |
        //         .                        |                        |
        //         V                        |                        |
        //         |                        |                        |
        //
        // When (X) occurrs, the receiver should not accept the message
        // from B since C has already resumed A as the next sender.
        //
        task_pool.set_src_tid(receiver, sender.tid());
    } else {
        task_pool.set_src_tid(receiver, src_tid);
    }
}

pub fn recv(
    task_pool: &mut TaskPool,
    src_tid: u32,
    message: &mut Message,
    flags: IpcFlags,
) -> KResult<()> {
    if src_tid == IpcSrcTask::ANY && task_pool.current().notifications().exists() {
        let current = task_pool.current();
        message.set_notification(current.notifications());
        task_pool.update_notifications(current, |_| Notifications::none());
    } else {
        if flags.is_noblock() {
            return KResult::WouldBlock;
        }

        let current = task_pool.current();
        resume_sender(task_pool, current, src_tid);
        task_pool.block_task(current);
        task_pool.task_switch();

        let current = task_pool.current();
        task_pool.update_message(current, |current_message| {
            mmio::memcpy_align4(message, current_message, 1)
        });
    }

    KResult::Ok(())
}

pub fn notify(
    task_pool: &mut TaskPool,
    dst_task: TaskRef,
    notifications: Notifications,
) -> KResult<()> {
    if dst_task.state() == TaskState::Blocked && dst_task.src_tid() == IpcSrcTask::ANY {
        // Send a NOTIFICATIONS message immediately.
        let dst_notifications = dst_task.notifications() | notifications;
        task_pool.update_message(dst_task, |dst_msg| {
            dst_msg.set_notification(dst_notifications)
        });
        task_pool.update_notifications(dst_task, |_| Notifications::none());
        task_pool.resume_task(dst_task);
    } else {
        // The task is not ready for receiving a event message: update the
        // pending notifications instead.
        task_pool.update_notifications(dst_task, |n| n | notifications);
    }
    KResult::Ok(())
}
