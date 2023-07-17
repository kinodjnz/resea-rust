use ::syscall::error::print_error;
use ::syscall::syscall;
use core::arch::global_asm;
use core::cell::Cell;
use core::mem;
use core::ptr;
use ipc::malloc::AllocMessage;
use klib::list;
use klib::macros::*;
use klib::result::KResult;

global_asm!(r#"
    .section .text.init
    .global malloc_task
malloc_task:
    auipc a0, %pcrel_hi(__malloc_task_stack_end)
    addi  sp, a0, %pcrel_lo(malloc_task)
    jump  {0}, t0
"#, sym malloc_task_rust);

fn malloc_task_rust() {
    let allocator: HeapAllocator = HeapAllocator::new();
    loop {
        match syscall::ipc_recv(0) {
            KResult::Ok(message) => {
                let payload = AllocMessage::parse_request(&message);
                let result = allocator.alloc(payload.size, payload.align, message.src_tid);
                let ptr = result.unwrap_or(ptr::null_mut());
                syscall::ipc_send(message.src_tid, &AllocMessage::response(ptr));
            }
            err => print_error!(b"ipc_recv failed: {}\n", err.err_as_u32()),
        };
    }
}

struct HeapAllocator {
    brk: Cell<*mut u32>,
    small_used: u32,
    small_free_chunks: [list::ListLink<SmallChunk>; Self::NUM_SMALL_CHUNKS],
    small_alloc_chunks: [list::ListLink<SmallChunk>; Self::NUM_TASKS],
}

#[repr(C)]
struct SmallChunk {
    size: Cell<usize>,
    link: list::ListLink<SmallChunk>,
    data: [u32; 0],
}

#[repr(C)]
struct LargestChunk {
    size: Cell<usize>,
    data: [u32; 0],
}

struct SmallChunkTag;

impl list::LinkAdapter<SmallChunkTag> for SmallChunk {
    fn link(&self) -> &list::ListLink<SmallChunk> {
        &self.link
    }
    fn from_link(link: &list::ListLink<SmallChunk>) -> &SmallChunk {
        unsafe {
            mem::transmute::<usize, &SmallChunk>(
                mem::transmute::<&list::ListLink<SmallChunk>, usize>(link) - 4, /*mem::offset_of!(SmallChunk, link)*/
            )
        }
    }
}

impl HeapAllocator {
    const NUM_TASKS: usize = 64;
    const MIN_ALIGN: usize = 8;
    const WORD_SIZE: usize = 4;
    const SMALL_CHUNK_SIZE_WORD: usize = mem::size_of::<SmallChunk>() / Self::WORD_SIZE;
    const NUM_SMALL_CHUNKS: usize = 32;
    const LARGE_CHUNK_MIN_REQ_SIZE: usize = 12 + 8 * 32;
    const LARGEST_CHUNK_SIZE_WORD: usize = mem::size_of::<LargestChunk>() / Self::WORD_SIZE;
    const ALLOCATED_BIT: usize = 2;
    const PREV_CHUNK_FREE_BIT: usize = 1;

    fn new() -> Self {
        let heap_start: usize = local_address_of!("__heap_start");
        let brk = ((heap_start | 4) + 4) as *mut u32;
        unsafe { *brk.sub(1) = 0 };
        Self {
            brk: Cell::new(brk),
            small_used: 0,
            small_free_chunks: zeroed_array!(list::ListLink<SmallChunk>, Self::NUM_SMALL_CHUNKS),
            small_alloc_chunks: zeroed_array!(list::ListLink<SmallChunk>, Self::NUM_TASKS),
        }
    }

    fn alloc(&self, size: usize, align: usize, tid: u32) -> KResult<*mut u8> {
        if align <= Self::MIN_ALIGN {
            self.alloc_unaligned(size, tid)
        } else {
            KResult::InvalidArg
        }
    }

    fn small_req_size_to_index(size: usize) -> usize {
        if size <= 4 {
            0
        } else {
            (size - 5) >> 3
        }
    }

    fn alloc_unaligned(&self, size: usize, tid: u32) -> KResult<*mut u8> {
        if size >= Self::LARGE_CHUNK_MIN_REQ_SIZE {
            self.alloc_unaligned_large(size, tid)
        } else {
            self.alloc_unaligned_small(size, tid)
        }
    }

    fn alloc_unaligned_large(&self, size: usize, _tid: u32) -> KResult<*mut u8> {
        let chunk_size_word =
            Self::LARGEST_CHUNK_SIZE_WORD + (size + Self::WORD_SIZE - 1) / Self::WORD_SIZE;
        let chunk: &mut LargestChunk = self.alloc_chunk(chunk_size_word);
        KResult::Ok(chunk.data.as_mut_ptr() as *mut u8)
    }

    fn alloc_chunk<T>(&self, size_word: usize) -> &'static mut T {
        unsafe {
            let chunk: &'static mut T = &mut *((self.brk.get().sub(1)) as *mut T);
            self.brk.update(|p| p.add(size_word));
            chunk
        }
    }

    fn list_for_small_free_chunks(
        &self,
        index: usize,
    ) -> list::LinkedList<'_, SmallChunk, SmallChunkTag> {
        list::LinkedList::new(&self.small_free_chunks[index])
    }

    fn list_for_small_alloc_chunks(
        &self,
        tid: u32,
    ) -> list::LinkedList<'_, SmallChunk, SmallChunkTag> {
        list::LinkedList::new(&self.small_alloc_chunks[tid as usize])
    }

    fn alloc_unaligned_small(&self, size: usize, tid: u32) -> KResult<*mut u8> {
        let index = Self::small_req_size_to_index(size);
        let index = (self.small_used & (0u32.wrapping_sub(1 << index))).trailing_zeros() as usize;
        let chunk: &'static SmallChunk = if index < 32 {
            self.list_for_small_free_chunks(index).pop_front().unwrap()
        } else {
            let chunk_size_word: usize =
                Self::SMALL_CHUNK_SIZE_WORD + (size + Self::WORD_SIZE - 1) / Self::WORD_SIZE;
            let chunk = self.alloc_chunk::<SmallChunk>(chunk_size_word);
            chunk.size.set(chunk_size_word * Self::WORD_SIZE);
            chunk
        };
        self.mark_allocated_small_chunk(chunk, tid);
        KResult::Ok(chunk.data.as_ptr() as *const u8 as *mut u8)
    }

    fn mark_allocated_small_chunk(&self, chunk: &'static SmallChunk, tid: u32) {
        chunk.size.update(|s| s | Self::ALLOCATED_BIT);
        self.list_for_small_alloc_chunks(tid).push_back(chunk);
    }
}
