use crate::bit_trie;
use crate::syscall::print_error;
use crate::syscall::syscall;
use core::cell::Cell;
use core::mem;
use core::ptr;
use ipc::malloc;
use klib::list::{self, RemovableLinkedStackOps};
use klib::result::KResult;
use klib::{local_address_of, zeroed_array};

#[no_mangle]
pub extern "C" fn malloc_task() {
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
    small_free_chunks: [list::RemovableStartLink<'static, SmallFreeChunk>; Self::NUM_SMALL_CHUNKS],
    large_free_chunks: [bit_trie::BitTrieRoot<'static, 4, LargeFreeChunk>; Self::NUM_LARGE_CHUNKS],
    alloc_chunks: [list::RemovableStartLink<'static, AllocChunk>; Self::NUM_TASKS],
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
struct Chunk {
    size: Cell<SizeField>,
    link: list::ListLink<'static, Chunk>,
    data: [u32; 0],
}

#[repr(C)]
struct AllocChunk {
    size: Cell<SizeField>,
    link: list::ListLink<'static, AllocChunk>,
    data: [u32; 0],
}

impl AllocChunk {
    fn shrink_chunk(&self, size_word: usize, new_size_word: usize) -> (&FreeChunk, usize) {
        let latter_chunk_size_word = size_word - new_size_word;
        let latter_chunk = unsafe {
            let chunk_ptr = mem::transmute::<&Self, *const u32>(&self);
            mem::transmute::<*const u32, &FreeChunk>(chunk_ptr.add(new_size_word))
        };
        latter_chunk.reset_with_prev_chunk_deallocated(latter_chunk_size_word);
        self.size.update(|s| s.with_size_word(new_size_word));
        (latter_chunk, latter_chunk_size_word)
    }

    fn init(&self, size_word: usize) {
        self.size
            .update(|s| s.with_size_word(size_word).with_allocated());
    }

    fn init_chunk_end_allocated_bit(&self, size_word: usize) {
        self.chunk_end_allocated_bit(size_word)
            .update(|s| s.with_prev_chunk_used());
    }

    fn chunk_end_allocated_bit(&self, size_word: usize) -> &Cell<SizeField> {
        let chunk_ptr: *const Cell<SizeField> = &self.size;
        unsafe { &*chunk_ptr.add(size_word) }
    }

    fn as_free_chunk(&self) -> &FreeChunk {
        unsafe { mem::transmute::<&Self, &FreeChunk>(&self) }
    }
}

#[repr(C)]
struct FreeChunk {
    size: Cell<SizeField>,
}

impl FreeChunk {
    fn init(&self, size_word: usize) {
        self.size
            .update(|s| s.with_size_word(size_word).with_deallocated());
    }

    fn reset_with_prev_chunk_deallocated(&self, size_word: usize) {
        self.size.update(|_| SizeField(size_word * WORD_SIZE));
        self.set_chunk_end_size(size_word);
        self.init_chunk_end_allocated_bit(size_word);
    }

    fn set_chunk_end_size(&self, size_word: usize) {
        unsafe {
            let chunk_ptr: *const Cell<u32> = mem::transmute::<&Self, *const Cell<u32>>(&self);
            (*chunk_ptr.add(size_word - 1)).set((size_word * WORD_SIZE) as u32);
        }
    }

    fn init_chunk_end_allocated_bit(&self, size_word: usize) {
        self.chunk_end_allocated_bit(size_word)
            .update(|s| s.with_prev_chunk_free());
    }

    fn chunk_end_allocated_bit(&self, size_word: usize) -> &Cell<SizeField> {
        let chunk_ptr: *const Cell<SizeField> = &self.size;
        unsafe { &*chunk_ptr.add(size_word) }
    }

    fn next_chunk_if_free(&self, size_word: usize) -> (Option<&FreeChunk>, usize) {
        unsafe {
            let chunk_ptr: *const Cell<SizeField> =
                mem::transmute::<&Self, *const Cell<SizeField>>(&self);
            let next_size_word_ptr = chunk_ptr.add(size_word);
            let size_field = (*next_size_word_ptr).get();
            let next_size_word = size_field.size_word();
            if next_size_word == 0 || size_field.allocated() {
                (None, 0)
            } else {
                (
                    Some(mem::transmute::<*const _, &FreeChunk>(next_size_word_ptr)),
                    next_size_word,
                )
            }
        }
    }

    fn prev_chunk_as_free(&self) -> (&FreeChunk, usize) {
        unsafe {
            let chunk_ptr: *const u32 = mem::transmute::<&Self, *const u32>(&self);
            let prev_chunk_size_word = (*chunk_ptr.sub(1) as usize) / WORD_SIZE;
            let prev_chunk: &FreeChunk =
                mem::transmute::<*const u32, &FreeChunk>(chunk_ptr.sub(prev_chunk_size_word));
            (prev_chunk, prev_chunk_size_word)
        }
    }

    fn as_sized(&self, size_word: usize) -> SizedFreeChunk {
        if size_word >= LARGE_CHUNK_MIN_SIZE_WORD {
            unsafe { SizedFreeChunk::Large(mem::transmute::<&Self, &LargeFreeChunk>(&self)) }
        } else {
            unsafe { SizedFreeChunk::Small(mem::transmute::<&Self, &SmallFreeChunk>(&self)) }
        }
    }
}

#[repr(C)]
struct SmallFreeChunk {
    size: Cell<SizeField>,
    link: list::ListLink<'static, SmallFreeChunk>,
}

impl SmallFreeChunk {
    fn as_alloc(&self) -> &AllocChunk {
        self.size.update(|s| s.with_allocated());
        unsafe { mem::transmute::<&Self, &AllocChunk>(&self) }
    }
}

#[repr(C)]
struct LargeFreeChunk {
    size: Cell<SizeField>,
    link: bit_trie::BitTrieLink<'static, 4, LargeFreeChunk>,
    data: [u32; 0],
}

enum SizedFreeChunk<'s> {
    Small(&'s SmallFreeChunk),
    Large(&'s LargeFreeChunk),
}

struct AllocChunkTag;
struct SmallFreeChunkTag;

const fn alloc_chunk_link_offset() -> usize {
    mem::offset_of!(AllocChunk, link)
}

const fn small_free_chunk_link_offset() -> usize {
    mem::offset_of!(SmallFreeChunk, link)
}

const fn alloc_chunk_data_offset_word() -> usize {
    mem::offset_of!(Chunk, data) / WORD_SIZE
}

const WORD_SIZE: usize = HeapAllocator::WORD_SIZE;
const LARGE_CHUNK_MIN_SIZE_WORD: usize = HeapAllocator::LARGE_CHUNK_MIN_SIZE_WORD;

impl list::LinkAdapter<'static, AllocChunkTag> for AllocChunk {
    fn link(&self) -> &list::ListLink<'static, AllocChunk> {
        &self.link
    }
    fn from_link<'a>(link: &'a list::ListLink<'static, AllocChunk>) -> &'a AllocChunk {
        unsafe {
            mem::transmute::<usize, &AllocChunk>(
                mem::transmute::<&list::ListLink<'static, AllocChunk>, usize>(link)
                    - alloc_chunk_link_offset(),
            )
        }
    }
}

impl list::LinkAdapter<'static, SmallFreeChunkTag> for SmallFreeChunk {
    fn link(&self) -> &list::ListLink<'static, SmallFreeChunk> {
        &self.link
    }
    fn from_link<'a>(link: &'a list::ListLink<'static, SmallFreeChunk>) -> &'a SmallFreeChunk {
        unsafe {
            mem::transmute::<usize, &SmallFreeChunk>(
                mem::transmute::<&list::ListLink<'static, SmallFreeChunk>, usize>(link)
                    - small_free_chunk_link_offset(),
            )
        }
    }
}

impl bit_trie::BitTrieLinkAdapter<'static, 4> for LargeFreeChunk {
    fn data(&self) -> usize {
        self.size.get().size_word()
    }

    fn from_bit_trie_link<'a>(link: &'a bit_trie::BitTrieLink<'static, 4, Self>) -> &'a Self {
        unsafe {
            mem::transmute::<usize, &Self>(
                mem::transmute::<&bit_trie::BitTrieLink<'static, 4, Self>, usize>(link)
                    - mem::offset_of!(LargeFreeChunk, link),
            )
        }
    }

    fn bit_trie_link(&self) -> &bit_trie::BitTrieLink<'static, 4, Self> {
        &self.link
    }
}

impl list::SingleLinkAdapter<'static, bit_trie::ChainTag> for LargeFreeChunk {
    fn link(&self) -> &list::SingleListLink<'static, Self> {
        bit_trie::BitTrieLinkAdapter::link(self)
    }

    fn from_link<'a>(link: &'a list::SingleListLink<'static, Self>) -> &'a Self {
        bit_trie::BitTrieLinkAdapter::from_link(link)
    }
}

impl HeapAllocator {
    const NUM_TASKS: usize = 64;
    const MIN_ALIGN: usize = 8;
    const WORD_SIZE: usize = 4;
    const CHUNK_SIZE_WORD: usize = mem::size_of::<Chunk>() / Self::WORD_SIZE;
    const MIN_CHUNK_SIZE_WORD: usize = Self::CHUNK_SIZE_WORD + 12;
    const NUM_SMALL_CHUNKS: usize = 32;
    const NUM_LARGE_CHUNKS: usize = 20;
    const LARGE_CHUNK_MIN_SIZE_WORD: usize =
        Self::CHUNK_SIZE_WORD + Self::LARGE_CHUNK_MIN_REQ_SIZE_WORD;
    const LARGE_CHUNK_MIN_REQ_SIZE_WORD: usize = 3 + 2 * 32;
    const LARGE_CHUNK_MIN_REQ_SIZE: usize = Self::LARGE_CHUNK_MIN_REQ_SIZE_WORD * Self::WORD_SIZE;
    const LARGE_CHUNK_SIZE_WORD: usize = mem::size_of::<LargeFreeChunk>() / Self::WORD_SIZE;

    const fn zeroed() -> Self {
        Self {
            brk: Cell::new(ptr::null_mut()), // points to the largest used heap address word.
            small_used: Cell::new(0),
            small_free_chunks: zeroed_array!(
                list::RemovableStartLink<SmallFreeChunk>,
                Self::NUM_SMALL_CHUNKS,
            ),
            large_free_chunks: zeroed_array!(
                bit_trie::BitTrieRoot<4, LargeFreeChunk>,
                Self::NUM_LARGE_CHUNKS,
            ),
            alloc_chunks: zeroed_array!(list::RemovableStartLink<AllocChunk>, Self::NUM_TASKS),
        }
    }

    fn init(&self) {
        let heap_start: u32 = local_address_of!("__heap_start");
        let brk = unsafe { (heap_start as *mut u32).sub(1) };
        unsafe { *brk = 0 };
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
            Self::LARGE_CHUNK_SIZE_WORD + (size + Self::WORD_SIZE - 1) / Self::WORD_SIZE;
        let chunk: &'static AllocChunk = self.alloc_chunk(chunk_size_word);
        chunk.init(chunk_size_word);
        chunk.init_chunk_end_allocated_bit(chunk_size_word);
        KResult::Ok(chunk.data.as_ptr() as *mut u8)
    }

    fn alloc_chunk<T>(&self, size_word: usize) -> &'static mut T {
        unsafe {
            let chunk: &'static mut T = &mut *(self.brk.get() as *mut T);
            self.brk.update(|p| {
                let brk = p.add(size_word);
                unsafe { *brk = 0 };
                brk
            });
            chunk
        }
    }

    fn dealloc(&self, ptr: *mut u8, tid: u32) {
        let chunk = self.ptr_to_alloc_chunk(ptr);
        self.list_for_alloc_chunks(tid).remove(chunk);
        self.free_and_combine(chunk);
    }

    fn ptr_to_alloc_chunk(&self, ptr: *mut u8) -> &'static AllocChunk {
        unsafe {
            let ptr = ptr as *mut u32;
            &*(ptr.sub(alloc_chunk_data_offset_word()) as *const AllocChunk)
        }
    }

    fn list_for_small_free_chunks(
        &self,
        index: usize,
    ) -> list::RemovableLinkedStack<'_, 'static, SmallFreeChunk, SmallFreeChunkTag> {
        list::RemovableLinkedStack::new(unsafe { &self.small_free_chunks.get_unchecked(index) })
    }

    fn list_for_alloc_chunks(
        &self,
        tid: u32,
    ) -> list::RemovableLinkedStack<'_, 'static, AllocChunk, AllocChunkTag> {
        list::RemovableLinkedStack::new(unsafe { &self.alloc_chunks.get_unchecked(tid as usize) })
    }

    fn alloc_unaligned_small(&self, size: usize, tid: u32) -> KResult<*mut u8> {
        let needed_chunk_size_word = Self::small_req_size_to_size_word(size);
        let index = Self::small_chunk_size_word_to_index(needed_chunk_size_word);
        let index =
            (self.small_used.get() & (0u32.wrapping_sub(1 << index))).trailing_zeros() as usize;
        let (chunk, chunk_size_word) = if index < 32 {
            let chunk = self.list_for_small_free_chunks(index).pop_front().unwrap();
            let chunk_size_word: usize = chunk.size.get().size_word();
            self.remove_from_small_free_chunks(chunk, chunk_size_word);
            let chunk = chunk.as_alloc();
            self.split_chunk_if_needed(chunk, chunk_size_word, needed_chunk_size_word)
        } else {
            let chunk: &'static AllocChunk = self.alloc_chunk::<AllocChunk>(needed_chunk_size_word);
            chunk.init(needed_chunk_size_word);
            (chunk, needed_chunk_size_word)
        };
        chunk.init_chunk_end_allocated_bit(chunk_size_word);
        self.list_for_alloc_chunks(tid).push_front(chunk);
        KResult::Ok(chunk.data.as_ptr() as *const u8 as *mut u8)
    }

    fn split_chunk_if_needed(
        &self,
        chunk: &'static AllocChunk,
        chunk_size_word: usize,
        needed_chunk_size_word: usize,
    ) -> (&'static AllocChunk, usize) {
        if chunk_size_word - needed_chunk_size_word >= Self::MIN_CHUNK_SIZE_WORD {
            let (next_chunk, next_chunk_size_word) =
                chunk.shrink_chunk(chunk_size_word, needed_chunk_size_word);
            match next_chunk.as_sized(next_chunk_size_word) {
                SizedFreeChunk::Small(chunk) => {
                    self.add_to_small_free_chunks(chunk, next_chunk_size_word)
                }
                SizedFreeChunk::Large(chunk) => {
                    self.add_to_large_free_chunks(chunk, next_chunk_size_word)
                }
            }
            (chunk, needed_chunk_size_word)
        } else {
            (chunk, chunk_size_word)
        }
    }

    fn remove_from_small_free_chunks(
        &self,
        chunk: &'static SmallFreeChunk,
        chunk_size_word: usize,
    ) {
        let index = Self::small_chunk_size_word_to_index(chunk_size_word);
        self.list_for_small_free_chunks(index).remove(chunk);
        if self.list_for_small_free_chunks(index).empty() {
            self.small_used.update(|u| u & !(1 << index));
        }
    }

    fn remove_from_large_free_chunks(
        &self,
        _chunk: &'static LargeFreeChunk,
        _chunk_size_word: usize,
    ) {
        // TODO implement
    }

    fn add_to_small_free_chunks(&self, chunk: &'static SmallFreeChunk, chunk_size_word: usize) {
        let index = Self::small_chunk_size_word_to_index(chunk_size_word);
        self.list_for_small_free_chunks(index).push_front(chunk);
        self.small_used.update(|u| u | (1 << index));
    }

    fn add_to_large_free_chunks(&self, chunk: &'static LargeFreeChunk, chunk_size_word: usize) {
        // tentative
        let chunk = unsafe { mem::transmute::<&LargeFreeChunk, &AllocChunk>(chunk) };
        let chunk_size_word = chunk.size.get().size_word();
        chunk.init(chunk_size_word);
        chunk.init_chunk_end_allocated_bit(chunk_size_word);

        // let index = Self::large_chunk_size_word_to_index(chunk_size_word);
        // self.list_for_small_free_chunks(index).push_front(chunk);
        // self.small_used.update(|u| u | (1 << index));
    }

    fn free_and_combine(&self, chunk: &'static AllocChunk) {
        let chunk = chunk.as_free_chunk();
        let size_field = chunk.size.get();
        let chunk_size_word = size_field.size_word();
        let next_chunk_size_word = if let (Some(next_chunk), next_chunk_size_word) =
            chunk.next_chunk_if_free(chunk_size_word)
        {
            match next_chunk.as_sized(next_chunk_size_word) {
                SizedFreeChunk::Small(chunk) => {
                    self.remove_from_small_free_chunks(chunk, next_chunk_size_word)
                }
                SizedFreeChunk::Large(chunk) => {
                    self.remove_from_large_free_chunks(chunk, next_chunk_size_word)
                }
            }
            next_chunk_size_word
        } else {
            0
        };
        let (chunk, prev_chunk_size_word) = if size_field.prev_chunk_free() {
            let (prev_chunk, prev_chunk_size_word) = chunk.prev_chunk_as_free();
            match prev_chunk.as_sized(prev_chunk_size_word) {
                SizedFreeChunk::Small(chunk) => {
                    self.remove_from_small_free_chunks(chunk, prev_chunk_size_word)
                }
                SizedFreeChunk::Large(chunk) => {
                    self.remove_from_large_free_chunks(chunk, prev_chunk_size_word)
                }
            }
            (prev_chunk, prev_chunk_size_word)
        } else {
            (chunk, 0)
        };
        let new_chunk_size_word = prev_chunk_size_word + chunk_size_word + next_chunk_size_word;
        chunk.init(new_chunk_size_word);
        chunk.set_chunk_end_size(new_chunk_size_word);
        chunk.init_chunk_end_allocated_bit(new_chunk_size_word);
        match chunk.as_sized(new_chunk_size_word) {
            SizedFreeChunk::Small(chunk) => {
                self.add_to_small_free_chunks(chunk, new_chunk_size_word)
            }
            SizedFreeChunk::Large(chunk) => {
                self.add_to_large_free_chunks(chunk, new_chunk_size_word)
            }
        }
    }
}
