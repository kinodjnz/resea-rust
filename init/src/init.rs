use ::syscall::error::print_error;
use alloc::alloc;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::{asm, global_asm};
use core::cell::Cell;
use core::ptr;
use core::slice;
use ipc::malloc::{AllocMessage, DeallocMessage};
use ipc::tid;
use klib::cycle;
use klib::ipc::{Message, MessageType};
use klib::result::KResult;
use syscall::syscall;

global_asm!(r#"
    .section .text.init
    .global init_task
init_task:
    auipc a0, %pcrel_hi(__init_task_stack_end)
    addi  sp, a0, %pcrel_lo(init_task)
    jump  {0}, t0
"#, sym init_task_rust);

global_asm!(r#"
    .section .text.init
    .global console_task
console_task:
    auipc a0, %pcrel_hi({0})
    lw    sp, %pcrel_lo(console_task)(a0)
    jump  {1}, t0
"#, sym CONSOLE_TASK_STACK, sym crate::generator::console_task);

global_asm!(r#"
    .section .text.init
    .global print1_task
print1_task:
    auipc a0, %pcrel_hi({0})
    lw    sp, %pcrel_lo(print1_task)(a0)
    jump  {1}, t0
"#, sym PRINT1_TASK_STACK, sym print1_task_rust);

static mut CONSOLE_TASK_STACK: Cell<*mut u8> = Cell::new(ptr::null_mut());
static mut PRINT1_TASK_STACK: Cell<*mut u8> = Cell::new(ptr::null_mut());

struct HeapAllocator;

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = syscall::ipc_call(
            tid::MALLOC_TASK_TID,
            &AllocMessage::request(layout.size(), layout.align()),
        );
        match result {
            KResult::Ok(response) => AllocMessage::parse_response(&response),
            err => {
                print_error!(b"alloc failed: {}\n", err.err_as_u32());
                ptr::null_mut()
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let result = syscall::ipc_call(tid::MALLOC_TASK_TID, &DeallocMessage::request(ptr));
        match result {
            KResult::Ok(_) => (),
            err => {
                print_error!(b"dealloc failed: {}\n", err.err_as_u32());
            }
        }
    }
}

#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator {};

pub fn init_task_rust() {
    cycle::init();
    syscall::console_write(b"init task started\n");
    let r = syscall::create_task(tid::MALLOC_TASK_TID, local_address_of!("malloc_task"));
    if r.is_err() {
        syscall::console_write(b"create malloc task failed\n");
    }
    unsafe {
        CONSOLE_TASK_STACK.set(alloc::alloc(Layout::from_size_align_unchecked(4096, 4)).add(4096))
    };
    let r = syscall::create_task(tid::CONSOLE_TASK_TID, local_address_of!("console_task"));
    if r.is_err() {
        syscall::console_write(b"create console task failed\n");
    }
    let next_user_task = tid::USER_TASK_START_TID;
    unsafe {
        PRINT1_TASK_STACK.set(alloc::alloc(Layout::from_size_align_unchecked(4096, 4)).add(4096))
    };
    let r = syscall::create_task(next_user_task, local_address_of!("print1_task"));
    if r.is_err() {
        syscall::console_write(b"create print1 task failed\n");
    }
    // next_user_task += 1;
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
        match syscall::ipc_send(tid::CONSOLE_TASK_TID, &message) {
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
        match syscall::ipc_send(tid::CONSOLE_TASK_TID, &message) {
            KResult::Ok(_) => (),
            err => print_error!(b"ipc_send failed: {}\n", err.err_as_u32()),
        };
        cycle::wait(cycle::clock_hz());
    }
}
