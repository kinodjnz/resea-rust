use crate::syscall;
use core::arch::{asm, global_asm};
use core::mem;
use core::slice;
use klib::cycle;
use klib::ipc::{Message, MessageType};
use klib::macros::*;
use klib::result::KResult;

const STACK_SIZE: usize = 4096;
const STACK_COUNT: usize = STACK_SIZE / 4;

#[repr(align(4096))]
pub struct UserStacks {
    _stack: [[u32; STACK_COUNT]; 3],
}

#[link_section = ".ubss"]
pub static mut USER_STACKS: UserStacks = UserStacks {
    _stack: [[0; STACK_COUNT]; 3],
};

global_asm!(r#"
    .section .text.init
    .global init_task
init_task:
    lla sp, {0} + {1}*1
    jump {2}, t0
"#, sym USER_STACKS, const STACK_SIZE, sym init_task_rust);

global_asm!(r#"
    .section .text.init
    .global console_task
console_task:
    lla sp, {0} + {1}*2
    jump {2}, t0
"#, sym USER_STACKS, const STACK_SIZE, sym crate::generator::console_task);

global_asm!(r#"
    .section .text.init
    .global print1_task
print1_task:
    lla sp, {0} + {1}*3
    jump {2}, t0
"#, sym USER_STACKS, const STACK_SIZE, sym print1_task_rust);

pub fn init_task_rust() {
    cycle::init();
    syscall::console_write(b"init task started\n");
    let r = syscall::create_task(2, local_address_of!("console_task"));
    if r.is_err() {
        syscall::console_write(b"create console task failed\n");
    }
    let r = syscall::create_task(3, local_address_of!("print1_task"));
    if r.is_err() {
        syscall::console_write(b"create print1 task failed\n");
    }
    print2_task()
}

pub struct ConsolePayload {
    data: *const u8,
    len: usize,
}

pub struct ConsoleMessage;

impl ConsoleMessage {
    pub const CONSOLE_OUT: MessageType = MessageType(2);

    pub fn text_of(message: &Message) -> &[u8] {
        unsafe {
            let payload: &ConsolePayload = &*(message.raw.as_ptr() as *const ConsolePayload);
            slice::from_raw_parts(payload.data, payload.len)
        }
    }

    pub fn new(payload: &[u8]) -> Message {
        Message {
            message_type: ConsoleMessage::CONSOLE_OUT,
            src_tid: 0,
            raw: unsafe {
                *(&ConsolePayload {
                    data: payload.as_ptr(),
                    len: payload.len(),
                } as *const ConsolePayload as *const [u8; 24])
            },
        }
    }
}

#[repr(align(4))]
pub struct AlignedVarArray<'a> {
    pub data: &'a [u8],
}

#[macro_export]
macro_rules! print_error {
    ($message:expr, $err:expr) => {
        print_error::<{ $message.len() + 8 }>($message, $err)
    };
}

pub fn print_error<const N: usize>(format: &[u8], err: u32) {
    let mut buf = [mem::MaybeUninit::uninit(); N];
    let mut writer = BufWriter::new(&mut buf);
    buf_fmt!(&mut writer, format, err);
    syscall::console_write(writer.as_slice());
}

pub fn console_task_rust() {
    syscall::console_write(b"console task started\n");
    loop {
        match syscall::ipc_recv(0) {
            KResult::Ok(message) => {
                syscall::console_write(ConsoleMessage::text_of(&message));
            }
            err => print_error!(b"ipc_recv failed: {}\n", err.err_as_u32()),
        };
    }
}

pub fn print1_task_rust() {
    syscall::console_write(b"print1 task started\n");
    loop {
        let message = ConsoleMessage::new(b"Hello, Resea\n");
        match syscall::ipc_send(2, &message) {
            KResult::Ok(_) => (),
            err => print_error!(b"ipc_send failed: {}\n", err.err_as_u32()),
        };
        cycle::wait(cycle::clock_hz());
    }
}

pub fn print2_task() {
    syscall::console_write(b"print2 task started\n");
    cycle::wait(cycle::clock_hz() / 2);
    loop {
        let message = ConsoleMessage::new(b"Hello, RISC-V\n");
        match syscall::ipc_send(2, &message) {
            KResult::Ok(_) => (),
            err => print_error!(b"ipc_send failed: {}\n", err.err_as_u32()),
        };
        cycle::wait(cycle::clock_hz());
    }
}
