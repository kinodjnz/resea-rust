use crate::macros::*;
use crate::syscall;
use crate::task;
use klib::ipc::{Message, MessageType};
use klib::result::KResult;
use klib::cycle;
use init::init::init_task;
use core::ptr;

pub fn kmain() -> ! {
    printk!(b"\nBooting Resea/Rust v0.0.1\n");
    cycle::init();
    if task::get_task_pool()
        .create_user_task(task::INIT_TID, (init_task as *const ()) as usize)
        .is_err()
    {
        printk!(b"create init task failed");
    }
    if task::get_task_pool()
        .create_user_task(2, (console_task as *const ()) as usize)
        .is_err()
    {
        printk!(b"create console task failed");
    }
    if task::get_task_pool()
        .create_user_task(3, (worker_task as *const ()) as usize)
        .is_err()
    {
        printk!(b"create worker task failed");
    }
    if task::get_task_pool().create_idle_task().is_err() {
        printk!(b"create idle task failed\n");
    }
    loop {}
}

struct UserMessage<T> {
    message_type: MessageType,
    src_tid: u32,
    payload: T,
}

impl <T> UserMessage<T> {
    fn from_message(message: &Message) -> &UserMessage<T> {
        unsafe {
            &*(message as *const Message as *const UserMessage<T>) // TODO size check
        }
    }

    fn as_message(&self) -> &Message {
        unsafe {
            &*(self as *const UserMessage<T> as *const Message)
        }
    }
}

struct ConsolePayload {
    data: *const u8,
    len: usize,
}

type ConsoleMessage = UserMessage<ConsolePayload>;

impl ConsoleMessage {
    fn new(text: &[u8]) -> ConsoleMessage {
        ConsoleMessage {
            message_type: MessageType(1),
            src_tid: 0,
            payload: ConsolePayload {
                data: text.as_ptr(),
                len: text.len(),
            }
        }
    }

    fn text(&self) -> &[u8] {
        unsafe {
            &*ptr::slice_from_raw_parts(self.payload.data, self.payload.len)
        }
    }
}

pub fn console_task() {
    printk!(b"console task started\n");
    loop {
        match syscall::ipc_recv(0) {
            KResult::Ok(message) => printk!(ConsoleMessage::from_message(&message).text()),
            err => {
                printk!(b"ipc_recv failed: {}", err.err_as_u32());
                return;
            }
        }
    }
}

pub fn worker_task() -> ! {
    cycle::wait(cycle::clock_hz() / 2);
    printk!(b"worker task started\n");
    loop {
        let message = ConsoleMessage::new(b"Hello, RISC-V\n");
        match syscall::ipc_send(2, &message.as_message()) {
            KResult::Ok(_) => (),
            err => printk!(b"ipc_send failed: {}", err.err_as_u32()),
        }
        cycle::wait(cycle::clock_hz());
    }
}
