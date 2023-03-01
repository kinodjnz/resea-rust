use klib::result::KResult;
use klib::ipc::{IpcFlags, Message};
use core::arch::asm;
use core::mem;

struct Syscall;

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