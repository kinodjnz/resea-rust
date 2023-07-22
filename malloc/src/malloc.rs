use ::syscall::error::print_error;
use ::syscall::syscall;
use core::arch::global_asm;
use core::cell::Cell;
use core::mem;
use core::ptr;
use ipc::malloc;
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
    let allocator: &HeapAllocator = unsafe { &HEAP_ALLOCATOR };
    allocator.init();

    loop {
        match syscall::ipc_recv(0) {
            KResult::Ok(message) => match message.message_type {
                malloc::ALLOC_MESSAGE => {
                    let payload = malloc::AllocMessage::parse_request(&message);
                    let result = allocator.alloc(payload.size, payload.align, message.src_tid);
                    let ptr = result.unwrap_or(ptr::null_mut());
                    syscall::ipc_send(message.src_tid, &malloc::AllocMessage::response(ptr));
                }
                malloc::DEALLOC_MESSAGE => {
                    let ptr = malloc::DeallocMessage::parse_request(&message);
                    allocator.dealloc(ptr, message.src_tid);
                    syscall::ipc_send(message.src_tid, &malloc::DeallocMessage::response());
                }
                _ => print_error!(b"unknown message type: {}\n", message.message_type.0),
            },
            err => print_error!(b"ipc_recv failed: {}\n", err.err_as_u32()),
        };
    }
}

#[repr(C)]
struct HeapAllocator {
    brk: Cell<*mut u32>,
    small_used: Cell<u32>,
    small_free_chunks: [list::ListLink<SmallChunk>; Self::NUM_SMALL_CHUNKS],
    small_alloc_chunks: [list::ListLink<SmallChunk>; Self::NUM_TASKS],
}

static mut HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::zeroed();

#[derive(Clone, Copy)]
struct SizeField(usize);

impl SizeField {
    const ALLOCATED_BIT: usize = 2;
    const PREV_CHUNK_FREE_BIT: usize = 1;
    const WORD_SIZE: usize = 4;

    fn with_size_word(&self, size: usize) -> Self {
        SizeField(
            (self.0 & (Self::ALLOCATED_BIT | Self::PREV_CHUNK_FREE_BIT)) | (size * Self::WORD_SIZE),
        )
    }

    fn size_word(&self) -> usize {
        self.0 / Self::WORD_SIZE
    }

    fn with_allocated(&self) -> Self {
        SizeField(self.0 | Self::ALLOCATED_BIT)
    }

    fn with_deallocated(&self) -> Self {
        SizeField(self.0 & !Self::ALLOCATED_BIT)
    }

    fn allocated(&self) -> bool {
        (self.0 & Self::ALLOCATED_BIT) != 0
    }

    fn with_prev_chunk_free(&self) -> Self {
        SizeField(self.0 | Self::PREV_CHUNK_FREE_BIT)
    }

    fn with_prev_chunk_used(&self) -> Self {
        SizeField(self.0 & !Self::PREV_CHUNK_FREE_BIT)
    }

    fn prev_chunk_free(&self) -> bool {
        (self.0 & Self::PREV_CHUNK_FREE_BIT) != 0
    }
}

#[repr(C)]
struct SmallChunk {
    size: Cell<SizeField>,
    link: list::ListLink<SmallChunk>,
    data: [u32; 0],
}

type Chunk = SmallChunk;

#[repr(C)]
struct LargestChunk {
    size: Cell<SizeField>,
    link: list::ListLink<SmallChunk>,
    data: [u32; 0],
}

trait AnyChunk {}

impl AnyChunk for SmallChunk {}

impl AnyChunk for LargestChunk {}

struct SmallChunkTag;

const SMALL_CHUNK_LINK_OFFSET: usize = 4; /*mem::offset_of!(SmallChunk, link)*/
const CHUNK_DATA_OFFSET_WORD: usize = 3; /*mem::offset_of!(Chunk, data) / WORD_SIZE */

impl list::LinkAdapter<SmallChunkTag> for SmallChunk {
    fn link(&self) -> &list::ListLink<SmallChunk> {
        &self.link
    }
    fn from_link(link: &list::ListLink<SmallChunk>) -> &SmallChunk {
        unsafe {
            mem::transmute::<usize, &SmallChunk>(
                mem::transmute::<&list::ListLink<SmallChunk>, usize>(link)
                    - SMALL_CHUNK_LINK_OFFSET,
            )
        }
    }
}

impl HeapAllocator {
    const NUM_TASKS: usize = 64;
    const MIN_ALIGN: usize = 8;
    const WORD_SIZE: usize = 4;
    const SMALL_CHUNK_SIZE_WORD: usize = mem::size_of::<SmallChunk>() / Self::WORD_SIZE;
    const MIN_CHUNK_SIZE_WORD: usize = Self::SMALL_CHUNK_SIZE_WORD + 12;
    const NUM_SMALL_CHUNKS: usize = 32;
    const LARGE_CHUNK_MIN_SIZE_WORD: usize =
        Self::SMALL_CHUNK_SIZE_WORD + Self::LARGE_CHUNK_MIN_REQ_SIZE_WORD;
    const LARGE_CHUNK_MIN_REQ_SIZE_WORD: usize = 3 + 2 * 32;
    const LARGE_CHUNK_MIN_REQ_SIZE: usize = Self::LARGE_CHUNK_MIN_REQ_SIZE_WORD * Self::WORD_SIZE;
    const LARGEST_CHUNK_SIZE_WORD: usize = mem::size_of::<LargestChunk>() / Self::WORD_SIZE;

    const fn zeroed() -> Self {
        Self {
            brk: Cell::new(ptr::null_mut()),
            small_used: Cell::new(0),
            small_free_chunks: zeroed_array!(list::ListLink<SmallChunk>, Self::NUM_SMALL_CHUNKS),
            small_alloc_chunks: zeroed_array!(list::ListLink<SmallChunk>, Self::NUM_TASKS),
        }
    }

    fn init(&self) {
        let heap_start: usize = local_address_of!("__heap_start");
        let brk = ((heap_start | 4) + 4) as *mut u32;
        unsafe { *brk.sub(1) = 0 };
        self.brk.set(brk);
    }

    fn alloc(&self, size: usize, align: usize, tid: u32) -> KResult<*mut u8> {
        if align <= Self::MIN_ALIGN {
            self.alloc_unaligned(size, tid)
        } else {
            KResult::InvalidArg
        }
    }

    fn small_req_size_to_size_word(size: usize) -> usize {
        if size <= 4 {
            6
        } else {
            6 + ((size - 5) >> 3 << 1)
        }
    }

    fn small_chunk_size_word_to_index(size_word: usize) -> usize {
        if size_word <= 6 {
            0
        } else {
            (size_word - 5) >> 1
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
        let chunk: &'static LargestChunk = self.alloc_chunk(chunk_size_word);
        self.mark_as_alloc_chunk(chunk, chunk_size_word);
        KResult::Ok(chunk.data.as_ptr() as *mut u8)
    }

    fn alloc_chunk<T>(&self, size_word: usize) -> &'static mut T {
        unsafe {
            let chunk: &'static mut T = &mut *((self.brk.get().sub(1)) as *mut T);
            self.brk.update(|p| p.add(size_word));
            chunk
        }
    }

    fn dealloc(&self, ptr: *mut u8, tid: u32) {
        let chunk = self.ptr_to_chunk(ptr);
        if chunk.size.get().size_word() >= Self::LARGE_CHUNK_MIN_SIZE_WORD {
            return;
        } else {
            self.dealloc_small(chunk, tid)
        }
    }

    fn ptr_to_chunk(&self, ptr: *mut u8) -> &'static Chunk {
        unsafe {
            let ptr = ptr as *mut u32;
            &*(ptr.sub(CHUNK_DATA_OFFSET_WORD) as *const Chunk)
        }
    }

    fn list_for_small_free_chunks(
        &self,
        index: usize,
    ) -> list::LinkedList<'_, SmallChunk, SmallChunkTag> {
        list::LinkedList::new(unsafe { &self.small_free_chunks.get_unchecked(index) })
    }

    fn list_for_small_alloc_chunks(
        &self,
        tid: u32,
    ) -> list::LinkedList<'_, SmallChunk, SmallChunkTag> {
        list::LinkedList::new(unsafe { &self.small_alloc_chunks.get_unchecked(tid as usize) })
    }

    fn alloc_unaligned_small(&self, size: usize, tid: u32) -> KResult<*mut u8> {
        let needed_chunk_size_word = Self::small_req_size_to_size_word(size);
        let index = Self::small_chunk_size_word_to_index(needed_chunk_size_word);
        let index =
            (self.small_used.get() & (0u32.wrapping_sub(1 << index))).trailing_zeros() as usize;
        let (chunk, chunk_size_word) = if index < 32 {
            let chunk = self.list_for_small_free_chunks(index).pop_front().unwrap();
            let chunk_size_word: usize = chunk.size.get().size_word();
            self.remove_from_free_chunks(chunk, chunk_size_word);
            let chunk_size_word = if chunk_size_word - needed_chunk_size_word
                >= Self::MIN_CHUNK_SIZE_WORD
            {
                let next_chunk_size_word = chunk_size_word - needed_chunk_size_word;
                let next_chunk = unsafe {
                    let chunk_ptr = mem::transmute::<&_, *const u32>(chunk);
                    mem::transmute::<*const u32, &SmallChunk>(chunk_ptr.add(needed_chunk_size_word))
                };
                next_chunk
                    .size
                    .update(|_| SizeField(next_chunk_size_word * Self::WORD_SIZE));
                self.set_free_size_word(next_chunk, next_chunk_size_word);
                self.mark_as_free_chunk(next_chunk, next_chunk_size_word);
                self.add_to_free_chunks(next_chunk, next_chunk_size_word);
                needed_chunk_size_word
            } else {
                chunk_size_word
            };
            chunk.size.update(|s| s.with_allocated());
            (chunk, chunk_size_word)
        } else {
            let chunk: &'static SmallChunk = self.alloc_chunk::<SmallChunk>(needed_chunk_size_word);
            chunk
                .size
                .update(|s| s.with_size_word(needed_chunk_size_word).with_allocated());
            (chunk, needed_chunk_size_word)
        };
        self.mark_as_alloc_chunk(chunk, chunk_size_word);
        self.list_for_small_alloc_chunks(tid).push_back(chunk);
        KResult::Ok(chunk.data.as_ptr() as *const u8 as *mut u8)
    }

    fn set_free_size_word(&self, chunk: &SmallChunk, size_word: usize) {
        unsafe {
            let chunk_ptr: *const Cell<u32> =
                mem::transmute::<&SmallChunk, *const Cell<u32>>(chunk);
            (*chunk_ptr.add(size_word - 1)).set((size_word * Self::WORD_SIZE) as u32);
        }
    }

    fn mark_as_free_chunk(&self, chunk: &'static SmallChunk, size_word: usize) {
        self.get_next_size_field(chunk, size_word)
            .update(|s| s.with_prev_chunk_free());
    }

    fn mark_as_alloc_chunk<Chunk: AnyChunk>(&self, chunk: &'static Chunk, size_word: usize) {
        self.get_next_size_field(chunk, size_word)
            .update(|s| s.with_prev_chunk_used());
    }

    fn get_next_size_field<Chunk: AnyChunk>(
        &self,
        chunk: &'static Chunk,
        size_word: usize,
    ) -> &Cell<SizeField> {
        unsafe {
            let chunk_ptr: *const Cell<SizeField> =
                mem::transmute::<&Chunk, *const Cell<SizeField>>(chunk);
            &*chunk_ptr.add(size_word)
        }
    }

    fn get_next_chunk(
        &self,
        chunk: &'static SmallChunk,
        size_word: usize,
    ) -> (Option<&'static SmallChunk>, usize) {
        unsafe {
            let chunk_ptr: *const Cell<SizeField> =
                mem::transmute::<&SmallChunk, *const Cell<SizeField>>(chunk);
            let next_size_word_ptr = chunk_ptr.add(size_word);
            let next_size_word = (*next_size_word_ptr).get().size_word();
            if next_size_word == 0 {
                (Option::None, 0)
            } else {
                (
                    Option::Some(
                        mem::transmute::<*const Cell<SizeField>, &'static SmallChunk>(
                            next_size_word_ptr,
                        ),
                    ),
                    next_size_word,
                )
            }
        }
    }

    fn get_prev_chunk(chunk: &SmallChunk) -> (&SmallChunk, usize) {
        unsafe {
            let chunk_ptr: *const u32 = mem::transmute::<&SmallChunk, *const u32>(chunk);
            let prev_chunk_size_word = (*chunk_ptr.sub(1) as usize) / Self::WORD_SIZE;
            let prev_chunk: &SmallChunk =
                mem::transmute::<*const u32, &SmallChunk>(chunk_ptr.sub(prev_chunk_size_word));
            (prev_chunk, prev_chunk_size_word)
        }
    }

    fn remove_from_free_chunks(&self, chunk: &'static SmallChunk, chunk_size_word: usize) {
        let index = Self::small_chunk_size_word_to_index(chunk_size_word);
        self.list_for_small_free_chunks(index).remove(chunk);
        if self.list_for_small_free_chunks(index).empty() {
            self.small_used.update(|u| u & !(1 << index));
        }
    }

    fn add_to_free_chunks(&self, chunk: &'static SmallChunk, chunk_size_word: usize) {
        let index = Self::small_chunk_size_word_to_index(chunk_size_word);
        self.list_for_small_free_chunks(index).push_back(chunk);
        self.small_used.update(|u| u | (1 << index));
    }

    fn free_and_combine(&self, chunk: &'static SmallChunk) {
        let chunk_size_word = chunk.size.get().size_word();
        let next_chunk_size_word = if let (Some(next_chunk), next_chunk_size_word) =
            self.get_next_chunk(chunk, chunk_size_word)
        {
            if next_chunk.size.get().allocated() {
                0
            } else {
                self.remove_from_free_chunks(next_chunk, next_chunk_size_word);
                next_chunk_size_word
            }
        } else {
            0
        };
        let (chunk, prev_chunk_size_word) = if chunk.size.get().prev_chunk_free() {
            let (prev_chunk, prev_chunk_size_word) = Self::get_prev_chunk(chunk);
            self.remove_from_free_chunks(prev_chunk, prev_chunk_size_word);
            (prev_chunk, prev_chunk_size_word)
        } else {
            (chunk, 0)
        };
        let new_chunk_size_word = prev_chunk_size_word + chunk_size_word + next_chunk_size_word;
        chunk
            .size
            .update(|s| s.with_size_word(new_chunk_size_word).with_deallocated());
        self.set_free_size_word(chunk, new_chunk_size_word);
        if new_chunk_size_word >= Self::LARGE_CHUNK_MIN_SIZE_WORD {
            // TODO: large chunk
            chunk.size.update(|s| s.with_allocated());
            self.mark_as_alloc_chunk(chunk, new_chunk_size_word);
        } else {
            self.mark_as_free_chunk(chunk, new_chunk_size_word);
            self.add_to_free_chunks(chunk, new_chunk_size_word);
        }
    }

    fn dealloc_small(&self, chunk: &'static SmallChunk, tid: u32) {
        self.list_for_small_alloc_chunks(tid).remove(chunk);
        self.free_and_combine(chunk);
    }
}
