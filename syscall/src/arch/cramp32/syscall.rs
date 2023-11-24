use ::klib::ipc::Message;
use ::klib::result::KResult;
use ::klib::syscall::Syscall;
use core::arch::asm;
use core::mem;

#[allow(dead_code)]
fn to_u32_result(a0: u32, a1: u32) -> KResult<u32> {
    if a0 == 0 {
        KResult::Ok(a1.into())
    } else {
        KResult::err_from_u32(a0)
    }
}

#[allow(dead_code)]
fn to_unit_result(a0: u32) -> KResult<()> {
    if a0 == 0 {
        KResult::Ok(Default::default())
    } else {
        KResult::err_from_u32(a0)
    }
}

#[allow(dead_code)]
pub fn syscall0r(syscall_id: Syscall) -> KResult<u32> {
    unsafe {
        let mut a0: u32;
        let mut a1: u32;
        asm!("ecall", in("a7") syscall_id as u32, out("a0") a0, out("a1") a1);
        to_u32_result(a0, a1)
    }
}

#[allow(dead_code)]
pub fn syscall0(syscall_id: Syscall) -> KResult<()> {
    unsafe {
        let mut a0: u32;
        asm!("ecall", in("a7") syscall_id as u32, out("a0") a0);
        to_unit_result(a0)
    }
}

#[allow(dead_code)]
pub fn syscall1(syscall_id: Syscall, mut a0: u32) -> KResult<()> {
    unsafe {
        asm!("ecall", inout("a0") a0, in("a7") syscall_id as u32);
        to_unit_result(a0)
    }
}

#[allow(dead_code)]
pub fn syscall2r(syscall_id: Syscall, mut a0: u32, mut a1: u32) -> KResult<u32> {
    unsafe {
        asm!("ecall", inout("a0") a0, inout("a1") a1, in("a7") syscall_id as u32);
        to_u32_result(a0, a1)
    }
}

#[allow(dead_code)]
pub fn syscall2(syscall_id: Syscall, mut a0: u32, a1: u32) -> KResult<()> {
    unsafe {
        asm!("ecall", inout("a0") a0, in("a1") a1, in("a7") syscall_id as u32);
        to_unit_result(a0)
    }
}

#[allow(dead_code)]
pub fn syscall4(syscall_id: Syscall, mut a0: u32, a1: u32, a2: u32, a3: u32) -> KResult<()> {
    unsafe {
        asm!("ecall", inout("a0") a0, in("a1") a1, in("a2") a2, in("a3") a3, in("a7") syscall_id as u32);
        to_unit_result(a0)
    }
}

pub fn nop() -> KResult<()> {
    syscall0(Syscall::Nop)
}

pub fn set_timer(timeout: u32) -> KResult<()> {
    syscall1(Syscall::SetTimer, timeout)
}

pub fn console_write(s: &[u8]) -> KResult<()> {
    syscall2(Syscall::ConsoleWrite, s.as_ptr() as u32, s.len() as u32)
}

pub fn ipc_recv(src_tid: u32) -> KResult<Message> {
    let mut message: mem::MaybeUninit<Message> = mem::MaybeUninit::uninit();
    syscall2(Syscall::IpcRecv, src_tid, unsafe {
        mem::transmute(<*mut _>::from(&mut message))
    })
    .map(|_| unsafe { message.assume_init() })
}

pub fn ipc_send(dst_tid: u32, message: &Message) -> KResult<()> {
    syscall2(Syscall::IpcSend, dst_tid, unsafe {
        mem::transmute(<*const _>::from(message))
    })
}

pub fn ipc_call(dst_tid: u32, message: &Message) -> KResult<Message> {
    let mut ipc_message: Message = *message;
    syscall2(Syscall::IpcCall, dst_tid, unsafe {
        mem::transmute(<*mut _>::from(&mut ipc_message))
    })
    .map(|_| ipc_message)
}

pub fn ipc_send_noblock(dst_tid: u32, message: &Message) -> KResult<()> {
    syscall2(Syscall::IpcSendNoblock, dst_tid, unsafe {
        mem::transmute(<*const _>::from(message))
    })
}

pub fn create_task(tid: u32, pc: usize) -> KResult<()> {
    syscall2(Syscall::CreateTask, tid, pc as u32)
}
