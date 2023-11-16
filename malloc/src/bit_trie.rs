use klib::list;
use core::cell::Cell;
use core::mem;

#[repr(C)]
pub struct BitTrieRoot<const BPL: usize, T: 'static> {
    root: Cell<Option<&'static BitTrieLink<BPL, T>>>,
    shift: usize,
}

impl<const BPL: usize, T: 'static> BitTrieRoot<BPL, T> {
    pub fn new(max_data_bits: usize) -> Self {
        Self {
            root: Cell::new(None),
            shift: max_data_bits - BPL,
        }
    }
}

pub struct ChainTag;

#[repr(C)]
pub struct BitTrieLink<const BPL: usize, T: 'static> {
    chain: list::SingleListLink<T>,
    parent: Cell<Option<&'static BitTrieLink<BPL, T>>>,
    children: [Cell<Option<&'static BitTrieLink<BPL, T>>>; BPL],
}

impl<const BPL: usize, T: 'static> BitTrieLink<BPL, T> {
    pub const fn init(children: [Cell<Option<&'static BitTrieLink<BPL, T>>>; BPL]) -> Self {
        Self {
            chain: list::SingleListLink::zeroed(),
            parent: Cell::new(Option::None),
            children,
        }
    }
}

#[repr(C)]
pub struct BitTrieChain<T: 'static> {
    list_link: list::SingleListLink<T>,
}

#[repr(C)]
pub struct ListLink<T: 'static> {
    list_link: list::ListLink<T>,
}

pub trait BitTrieLinkAdapter<const BPL: usize>: list::SingleLinkAdapter<ChainTag> {
    fn link(&self) -> &list::SingleListLink<Self>
    where
        Self: Sized
    {
        unsafe {
            &mem::transmute::<&BitTrieLink<BPL, Self>, &BitTrieChain<Self>>(self.bit_trie_link()).list_link
        }
    }
    fn from_link(link: &list::SingleListLink<Self>) -> &Self
    where
        Self: Sized
    {
        unsafe {
            Self::from_bit_trie_link(mem::transmute::<&list::SingleListLink<Self>, &BitTrieLink<BPL, Self>>(link))
        }
    }

    fn data(&self) -> usize;
    fn bit_trie_link(&self) -> &BitTrieLink<BPL, Self>
    where
        Self: Sized;
    fn from_bit_trie_link(link: &BitTrieLink<BPL, Self>) -> &Self
    where
        Self: Sized;
}

const BIT_TRIE_LINK_OFFSET: usize = 4; /*mem::offset_of!(T, list_link)*/

impl<const BPL: usize, T: 'static> BitTrieRoot<BPL, T>
where T: BitTrieLinkAdapter<BPL> {
    pub fn unlink_lowest(&self) -> Option<&'static T> {
        self.unlink_lowest_in_subtree(self.root.get(), 0, usize::MAX, None, 0)
    }

    pub fn unlink_eq_or_above(&self, data: usize) -> Option<&'static T> {
        let mut shift = self.shift;
        let mut ptr = self.root.get();
        let mut nearest_above_data = usize::MAX;
        let mut nearest_above = None;
        let mut nearest_above_parents_index = 0;
        let mut next_above_link = None;
        let mut next_above_parents_index = 0;
        let mut parents_index = 0;
        while let Some(cur) = ptr {
            let cur_chunk = T::from_bit_trie_link(cur);
            if cur_chunk.data() == data {
                self.unlink_chunk(cur, parents_index);
                return Some(cur_chunk);
            } else if data < cur_chunk.data() && cur_chunk.data() < nearest_above_data {
                nearest_above_data = cur_chunk.data();
                nearest_above = Some(cur);
                nearest_above_parents_index = parents_index;
            }
            let index = (data >> shift) & ((1 << BPL) - 1);
            ptr = unsafe { cur.children.get_unchecked(index).get() };
            for i in index + 1..(1 << BPL) {
                if let Some(link) = unsafe { cur.children.get_unchecked(i).get() } {
                    next_above_link = Some(link);
                    next_above_parents_index = i;
                    break;
                }
            }
            parents_index = index;
            shift -= 2;
        }
        self.unlink_lowest_in_subtree(next_above_link, next_above_parents_index, nearest_above_data, nearest_above, nearest_above_parents_index)
    }

    fn unlink_lowest_in_subtree(&self, ptr: Option<&'static BitTrieLink<BPL, T>>, parents_index: usize, nearest_above_data: usize, nearest_above: Option<&'static BitTrieLink<BPL, T>>, nearest_above_parents_index: usize) -> Option<&'static T> {
        let mut ptr_index = ptr.map(|p| (p, parents_index));
        let mut nearest_above_data = nearest_above_data;
        let mut nearest_above = nearest_above;
        let mut nearest_above_parents_index = nearest_above_parents_index;
        while let Some((cur, index)) = ptr_index {
            let cur_chunk = T::from_bit_trie_link(cur);
            if cur_chunk.data() < nearest_above_data {
                nearest_above_data = cur_chunk.data();
                nearest_above = Some(cur);
                nearest_above_parents_index = index;
            }
            ptr_index = (0..(1 << BPL))
                .find_map(|i| unsafe { cur.children.get_unchecked(i).get() }.map(|link| (link, i)) );
        }
        nearest_above.map(|link| {
            self.unlink_chunk(link, nearest_above_parents_index);
            T::from_bit_trie_link(link)
        })
    }

    fn unlink_chunk(&self, link: &'static BitTrieLink<BPL, T>, parents_index: usize) {
        if let Some(next) = self.list_for_data(link).pop_front() {
            self.replace_chunk(link, next.bit_trie_link(), parents_index);
        } else {
            let mut ptr_index = Some((link, parents_index));
            let mut last_link = link;
            let mut last_index = parents_index;
            while let Some((cur, index)) = ptr_index {
                last_link = cur;
                last_index = index;
                ptr_index = (0..(1 << BPL))
                    .rev()
                    .find_map(|i| unsafe { cur.children.get_unchecked(i).get() }.map(|link| (link, i)));
            }
            self.replace_chunk(link, last_link, last_index);
        }
    }

    fn replace_chunk(&self, cur: &'static BitTrieLink<BPL, T>, replaced: &'static BitTrieLink<BPL, T>, parents_index: usize) {
        replaced.parent.set(cur.parent.get());
        if let Some(parent) = cur.parent.get() {
            unsafe { parent.children.get_unchecked(parents_index) }.set(Some(replaced));
        } else {
            self.root.set(Some(replaced));
        }
        for i in 0..(1 << BPL) {
            unsafe {
                replaced.children.get_unchecked(i).set(cur.children.get_unchecked(i).get());
                cur.children.get_unchecked(i).get().map(|child| child.parent.set(Some(replaced)));
            }
        }
    }

    fn list_for_data(&self, link: &'static BitTrieLink<BPL, T>) -> list::LinkedStack<'_, T, ChainTag> {
        list::LinkedStack::new(&link.chain)
    }

    pub fn insert(&self, chunk: &'static T) {
        let mut shift = self.shift;
        let mut ptr = &self.root;
        let mut parent = None;
        while let Some(cur) = ptr.get() {
            let cur_chunk = T::from_bit_trie_link(cur);
            if cur_chunk.data() == chunk.data() {
                self.list_for_data(cur).push_front(chunk);
                return;
            }
            let index = (chunk.data() >> shift) & ((1 << BPL) - 1);
            ptr = unsafe { &cur.children.get_unchecked(index) };
            parent = Some(cur);
            shift -= 2;
        }
        ptr.set(Some(chunk.bit_trie_link()));
        chunk.bit_trie_link().parent.set(parent);
    }
}
