use crate::arch;
use klib::ipc::Message;
use klib::result::KResult;

pub fn nop() -> KResult<()> {
    arch::syscall::nop()
}

pub fn set_timer(timeout: u32) -> KResult<()> {
    arch::syscall::set_timer(timeout)
}

pub fn console_write(s: &[u8]) -> KResult<()> {
    arch::syscall::console_write(s)
}

pub fn ipc_recv(src_tid: u32) -> KResult<Message> {
    arch::syscall::ipc_recv(src_tid)
}

pub fn ipc_send(dst_tid: u32, message: &Message) -> KResult<()> {
    arch::syscall::ipc_send(dst_tid, message)
}

pub fn ipc_call(dst_tid: u32, message: &Message) -> KResult<Message> {
    arch::syscall::ipc_call(dst_tid, message)
}

pub fn ipc_send_noblock(dst_tid: u32, message: &Message) -> KResult<()> {
    arch::syscall::ipc_send_noblock(dst_tid, message)
}

pub fn create_task(tid: u32, pc: usize) -> KResult<()> {
    arch::syscall::create_task(tid, pc)
}
