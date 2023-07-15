use ::syscall::error::print_error;
use ::syscall::syscall;
use core::arch::global_asm;
use core::cell::Cell;
use core::mem;
use core::ptr;
use ipc::malloc::AllocMessage;
use klib::macros::*;
use klib::result::KResult;

global_asm!(r#"
    .section .text.init
    .global malloc_task
malloc_task:
    lla sp, __malloc_task_stack_end
    jump {0}, t0
"#, sym malloc_task_rust);

fn malloc_task_rust() {
    let allocator: HeapAllocator = HeapAllocator::new();
    loop {
        match syscall::ipc_recv(0) {
            KResult::Ok(message) => {
                let payload = AllocMessage::parse_request(&message);
                let result = allocator.alloc(payload.size, payload.align);
                let ptr = result.unwrap_or(ptr::null_mut());
                syscall::ipc_send(message.src_tid, &AllocMessage::response(ptr));
            }
            err => print_error!(b"ipc_recv failed: {}\n", err.err_as_u32()),
        };
    }
}

struct HeapAllocator {
    brk: Cell<*mut u32>,
}

#[repr(C)]
struct SmallChunk {
    size: Cell<usize>,
    forward: Cell<*mut u32>,
    backward: Cell<*mut u32>,
    data: [u32; 0],
}

impl HeapAllocator {
    const MIN_ALIGN: usize = 8;
    const WORD_SIZE: usize = 4;
    const SMALL_CHUNK_SIZE_WORD: usize = mem::size_of::<SmallChunk>() / Self::WORD_SIZE;

    pub fn new() -> Self {
        let heap_start: usize = local_address_of!("__heap_start");
        Self {
            brk: Cell::new((heap_start | 4) as *mut u32),
        }
    }

    pub fn alloc(&self, size: usize, align: usize) -> KResult<*mut u8> {
        if align <= Self::MIN_ALIGN {
            self.alloc_unaligned(size)
        } else {
            KResult::InvalidArg
        }
    }

    fn alloc_unaligned(&self, size: usize) -> KResult<*mut u8> {
        unsafe {
            let chunk: &mut SmallChunk = &mut *(self.brk.get() as *mut SmallChunk);
            self.brk
                .update(|p| p.add(Self::SMALL_CHUNK_SIZE_WORD + size / Self::WORD_SIZE));
            KResult::Ok(chunk.data.as_mut_ptr() as *mut u8)
        }
    }
}
