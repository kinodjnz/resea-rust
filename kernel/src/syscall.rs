use crate::config;
use crate::console::Console;
use crate::ipc;
use crate::task;
use core::slice;
use klib::ipc::{IpcFlags, Message, Notifications};
use klib::result::KResult;

pub struct Syscall;

#[allow(unused)]
impl Syscall {
    pub const NOP: u32 = 1;
    pub const KDEBUG: u32 = 2;
    pub const IPC: u32 = 3;
    pub const NOTIFY: u32 = 4;
    pub const SET_TIMER: u32 = 5;
    pub const CONSOLE_WRITE: u32 = 6;
    pub const CREATE_TASK: u32 = 8;
    pub const DESTROY_TASK: u32 = 9;
    pub const EXIT_TASK: u32 = 10;
    pub const TASK_SELF: u32 = 11;
    pub const SCHEDULE_TASK: u32 = 12;
    pub const IRQ_ACQUIRE: u32 = 15;
    pub const IRQ_RELEASE: u32 = 16;
}

fn handle_set_timer(timeout: u32) -> KResult<()> {
    let task_pool = task::get_task_pool();
    task_pool.set_current_timeout(timeout)
}

fn handle_console_write(s: &[u8]) -> KResult<()> {
    if s.len() > 1024 {
        KResult::TooLarge
    } else {
        Console::puts(s);
        KResult::Ok(())
    }
}

// Send/receive IPC messages.
fn handle_ipc(dst_tid: u32, src_tid: u32, message: &mut Message, flags: IpcFlags) -> KResult<()> {
    if flags.is_kernel() {
        return KResult::InvalidArg;
    }
    if src_tid > config::NUM_TASKS {
        return KResult::InvalidArg;
    }

    let task_pool = task::get_task_pool();
    let result = if flags.is_send() {
        task_pool
            .lookup_task(dst_tid)
            .and_then(|task| ipc::send(task_pool, task, message, flags))
    } else {
        KResult::Ok(())
    };
    if flags.is_recv() {
        let recv_flags = if flags.is_send() {
            // In case of nonblocking ipc call, the destination should respond soon.
            flags.clear_noblock()
        } else {
            flags
        };
        result.and_then(|_| ipc::recv(task_pool, src_tid, message, recv_flags))
    } else {
        result
    }
}

// Sends notifications.
fn handle_notify(dst_tid: u32, notifications: Notifications) -> KResult<()> {
    let task_pool = task::get_task_pool();
    task_pool
        .lookup_task(dst_tid)
        .and_then(|task| ipc::notify(task_pool, task, notifications))
}

fn handle_create_task(tid: u32, pc: usize) -> KResult<()> {
    let task_pool = task::get_task_pool();
    return task_pool.create_user_task(tid, pc);
}

#[no_mangle]
pub extern "C" fn handle_syscall(
    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    _a4: u32,
    _a5: u32,
    _syscall_subid: u32,
    syscall_id: u32,
) -> u32 {
    let r = match syscall_id {
        Syscall::NOP => KResult::Ok(()),
        Syscall::SET_TIMER => handle_set_timer(a0),
        Syscall::CONSOLE_WRITE => {
            handle_console_write(unsafe { slice::from_raw_parts(a0 as *const u8, a1 as usize) })
        }
        Syscall::IPC => handle_ipc(
            a0,
            a1,
            unsafe { &mut *(a2 as *mut Message) },
            IpcFlags::from_u32(a3),
        ),
        Syscall::NOTIFY => handle_notify(a0, Notifications::from_u32(a1)),
        Syscall::CREATE_TASK => handle_create_task(a0, a1 as usize),
        _ => KResult::InvalidArg,
    };
    match r {
        KResult::Ok(()) => 0,
        e => e.err_as_u32(),
    }
}
