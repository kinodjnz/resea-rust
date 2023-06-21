use crate::syscall;
use core::mem;
use core::ptr;
use klib::cycle;
use klib::ipc::{Message, MessageType};
use klib::macros::*;
use klib::result::KResult;

#[no_mangle]
pub extern "C" fn init_task() {
    cycle::init();
    syscall::console_write(b"init task started\n");
    let r = syscall::create_task(2, (console_task as *const ()) as usize);
    if r.is_err() {
        syscall::console_write(b"create console task failed\n");
    }
    let r = syscall::create_task(3, (print1_task as *const ()) as usize);
    if r.is_err() {
        syscall::console_write(b"create print1 task failed\n");
    }
    print2_task()
}

#[allow(unused)]
struct UserMessage<T> {
    message_type: MessageType,
    src_tid: u32,
    payload: T,
}

impl<T> UserMessage<T> {
    fn from_message(message: &Message) -> &UserMessage<T> {
        unsafe {
            &*(message as *const Message as *const UserMessage<T>) // TODO size check
        }
    }

    fn as_message(&self) -> &Message {
        unsafe { &*(self as *const UserMessage<T> as *const Message) }
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
            },
        }
    }

    fn text(&self) -> &[u8] {
        unsafe { &*ptr::slice_from_raw_parts(self.payload.data, self.payload.len) }
    }
}

macro_rules! print_error {
    ($message:expr, $err:expr) => {
        print_error::<{ $message.len() + 8 }>($message, $err)
    };
}

fn print_error<const N: usize>(format: &[u8], err: u32) {
    let mut buf = [mem::MaybeUninit::uninit(); N];
    let mut writer = BufWriter::new(&mut buf);
    buf_fmt!(&mut writer, format, err);
    syscall::console_write(writer.as_slice());
}

pub fn console_task() {
    syscall::console_write(b"console task started\n");
    loop {
        match syscall::ipc_recv(0) {
            KResult::Ok(message) => {
                syscall::console_write(ConsoleMessage::from_message(&message).text());
            }
            err => print_error!(b"ipc_recv failed: {}\n", err.err_as_u32()),
        };
    }
}

pub fn print1_task() {
    syscall::console_write(b"print1 task started\n");
    loop {
        let message = ConsoleMessage::new(b"Hello, Resea\n");
        match syscall::ipc_send(2, &message.as_message()) {
            KResult::Ok(_) => (),
            err => print_error!(b"ipc_send failed: {}\n", err.err_as_u32()),
        };
        cycle::wait(cycle::clock_hz());
    }
}

pub fn print2_task() {
    cycle::wait(cycle::clock_hz() / 2);
    syscall::console_write(b"print2 task started\n");
    loop {
        let message = ConsoleMessage::new(b"Hello, RISC-V\n");
        match syscall::ipc_send(2, &message.as_message()) {
            KResult::Ok(_) => (),
            err => print_error!(b"ipc_send failed: {}\n", err.err_as_u32()),
        };
        cycle::wait(cycle::clock_hz());
    }
}
