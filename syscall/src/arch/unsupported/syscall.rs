use ::klib::ipc::Message;
use ::klib::result::KResult;

pub fn nop() -> KResult<()> {
    unimplemented!();
}

pub fn set_timer(_timeout: u32) -> KResult<()> {
    unimplemented!();
}

pub fn console_write(_s: &[u8]) -> KResult<()> {
    unimplemented!();
}

pub fn ipc_recv(_src_tid: u32) -> KResult<Message> {
    unimplemented!();
}

pub fn ipc_send(_dst_tid: u32, _message: &Message) -> KResult<()> {
    unimplemented!();
}

pub fn ipc_call(_dst_tid: u32, _message: &Message) -> KResult<Message> {
    unimplemented!();
}

pub fn ipc_send_noblock(_dst_tid: u32, _message: &Message) -> KResult<()> {
    unimplemented!();
}

pub fn create_task(_tid: u32, _pc: u32, _sp: u32) -> KResult<()> {
    unimplemented!();
}
