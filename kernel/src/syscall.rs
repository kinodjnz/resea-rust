use crate::config;
use crate::console::Console;
use crate::ipc::{self, IpcFlags};
use crate::result::KResult;
use crate::task::{self, Message, Notifications};
use core::arch::asm;
use core::mem;
use core::slice;

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
        result.and_then(|_| ipc::recv(task_pool, src_tid, message, flags))
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
        _ => KResult::InvalidArg,
    };
    match r {
        KResult::Ok(()) => 0,
        e => e.err_as_u32(),
    }
}

#[allow(unused)]
fn to_u32_result(a0: u32, a1: u32) -> KResult<u32> {
    if a0 == 0 {
        KResult::Ok(a1.into())
    } else {
        KResult::err_from_u32(a0)
    }
}

#[allow(unused)]
fn to_unit_result(a0: u32) -> KResult<()> {
    if a0 == 0 {
        KResult::Ok(Default::default())
    } else {
        KResult::err_from_u32(a0)
    }
}

#[allow(unused)]
pub fn syscall0r(syscall_id: u32) -> KResult<u32> {
    unsafe {
        let mut a0: u32;
        let mut a1: u32;
        asm!("ecall", in("a7") syscall_id, out("a0") a0, out("a1") a1);
        to_u32_result(a0, a1)
    }
}

#[allow(unused)]
pub fn syscall0(syscall_id: u32) -> KResult<()> {
    unsafe {
        let mut a0: u32;
        asm!("ecall", in("a7") syscall_id, out("a0") a0);
        to_unit_result(a0)
    }
}

#[allow(unused)]
pub fn syscall2r(syscall_id: u32, mut a0: u32, mut a1: u32) -> KResult<u32> {
    unsafe {
        asm!("ecall", inout("a0") a0, inout("a1") a1, in("a7") syscall_id);
        to_u32_result(a0, a1)
    }
}

#[allow(unused)]
pub fn syscall2(syscall_id: u32, mut a0: u32, mut a1: u32) -> KResult<()> {
    unsafe {
        asm!("ecall", inout("a0") a0, inout("a1") a1, in("a7") syscall_id);
        to_unit_result(a0)
    }
}

#[allow(unused)]
pub fn syscall4(
    syscall_id: u32,
    mut a0: u32,
    mut a1: u32,
    mut a2: u32,
    mut a3: u32,
) -> KResult<()> {
    unsafe {
        asm!("ecall", inout("a0") a0, inout("a1") a1, inout("a2") a2, inout("a3") a3, in("a7") syscall_id);
        to_unit_result(a0)
    }
}

#[allow(unused)]
pub fn nop() -> KResult<()> {
    syscall0(Syscall::NOP)
}

#[allow(unused)]
pub fn console_write(s: &[u8]) -> KResult<()> {
    syscall2(Syscall::CONSOLE_WRITE, s.as_ptr() as u32, s.len() as u32)
}

#[allow(unused)]
pub fn ipc_recv(dst_tid: u32) -> KResult<Message> {
    let mut message: Message = unsafe { mem::MaybeUninit::zeroed().assume_init() };
    syscall4(
        Syscall::IPC,
        dst_tid,
        0,
        unsafe { mem::transmute::<*mut Message, u32>(&mut message as *mut Message) },
        IpcFlags::recv().as_u32(),
    ).map(|_| message)
}

#[allow(unused)]
pub fn ipc_send(dst_tid: u32, message: &Message) -> KResult<()> {
    syscall4(
        Syscall::IPC,
        dst_tid,
        0,
        unsafe { mem::transmute::<*const Message, u32>(message as *const Message) },
        IpcFlags::send().as_u32(),
    )
}
