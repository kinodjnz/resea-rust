use crate::config;
use crate::console::Console;
use crate::ipc;
use crate::task;
use core::mem;
use core::slice;
use klib::ipc::{IpcFlags, Message, Notifications};
use klib::result::KResult;
use klib::syscall::Syscall;

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

fn handle_ipc_send(dst_tid: u32, message: &mut Message) -> KResult<()> {
    let task_pool = task::get_task_pool();
    task_pool
        .lookup_task(dst_tid)
        .and_then(|task| ipc::send(task_pool, task, message, IpcFlags::block()))
}

fn handle_ipc_recv(src_tid: u32, message: &mut Message) -> KResult<()> {
    if src_tid > config::NUM_TASKS {
        return KResult::InvalidArg;
    }

    let task_pool = task::get_task_pool();
    ipc::recv(task_pool, src_tid, message, IpcFlags::block())
}

fn handle_ipc_call(dst_tid: u32, message: &mut Message) -> KResult<()> {
    let task_pool = task::get_task_pool();
    task_pool
        .lookup_task(dst_tid)
        .and_then(|task| ipc::send(task_pool, task, message, IpcFlags::block()))
        .and_then(|_| ipc::recv(task_pool, dst_tid, message, IpcFlags::block()))
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
    _a2: u32,
    _a3: u32,
    _a4: u32,
    _a5: u32,
    _syscall_subid: u32,
    syscall_id: u32,
) -> u32 {
    let r = match syscall_id {
        i if i == Syscall::Nop.as_u32() => KResult::Ok(()),
        i if i == Syscall::SetTimer.as_u32() => handle_set_timer(a0),
        i if i == Syscall::ConsoleWrite.as_u32() => {
            handle_console_write(unsafe { slice::from_raw_parts(a0 as *const u8, a1 as usize) })
        }
        i if i == Syscall::IpcSend.as_u32() => {
            handle_ipc_send(a0, unsafe { mem::transmute::<u32, &mut Message>(a1) })
        }
        i if i == Syscall::IpcRecv.as_u32() => {
            handle_ipc_recv(a0, unsafe { mem::transmute::<u32, &mut Message>(a1) })
        }
        i if i == Syscall::IpcCall.as_u32() => {
            handle_ipc_call(a0, unsafe { mem::transmute::<u32, &mut Message>(a1) })
        }
        i if i == Syscall::Notify.as_u32() => handle_notify(a0, Notifications::from_u32(a1)),
        i if i == Syscall::CreateTask.as_u32() => handle_create_task(a0, a1 as usize),
        _ => KResult::InvalidArg,
    };
    match r {
        KResult::Ok(()) => 0,
        e => e.err_as_u32(),
    }
}
