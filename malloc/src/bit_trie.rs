use klib::list;
use core::cell::Cell;
use core::mem;

#[repr(C)]
pub struct BitTrieRoot<'s, const NCPL: usize, T: 's> {
    // NCPL: Number of Children Per Link
    root: Cell<Option<&'s BitTrieLink<'s, NCPL, T>>>,
    shift: usize,
}

impl<'s, const NCPL: usize, T: 's> BitTrieRoot<'s, NCPL, T> {
    pub fn new(max_data_bits: usize) -> Self {
        Self {
            root: Cell::new(None),
            shift: max_data_bits - NCPL.ilog2() as usize,
        }
    }
}

pub struct ChainTag;

#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub struct BitTrieLink<'s, const NCPL: usize, T: 's> {
    chain: list::SingleListLink<'s, T>,
    parent: Cell<Option<&'s BitTrieLink<'s, NCPL, T>>>,
    children: [Cell<Option<&'s BitTrieLink<'s, NCPL, T>>>; NCPL],
}

impl<'s, const NCPL: usize, T: 's> BitTrieLink<'s, NCPL, T> {
    pub const fn init(children: [Cell<Option<&'s BitTrieLink<'s, NCPL, T>>>; NCPL]) -> Self {
        Self {
            chain: list::SingleListLink::zeroed(),
            parent: Cell::new(Option::None),
            children,
        }
    }

    fn clear_trie_link(&self) {
        self.parent.set(None);
        for i in 0..NCPL {
            unsafe {
                self.children.get_unchecked(i).set(None);
            }
        }
    }
}

#[repr(C)]
pub struct BitTrieChain<'s, T: 's> {
    list_link: list::SingleListLink<'s, T>,
}

#[repr(C)]
pub struct ListLink<'s, T: 's> {
    list_link: list::ListLink<'s, T>,
}

pub trait BitTrieLinkAdapter<'s, const NCPL: usize>: list::SingleLinkAdapter<'s, ChainTag> {
    fn link(&self) -> &list::SingleListLink<'s, Self>
    where
        Self: Sized
    {
        unsafe {
            &mem::transmute::<&BitTrieLink<'s, NCPL, Self>, &BitTrieChain<'s, Self>>(self.bit_trie_link()).list_link
        }
    }
    fn from_link<'a>(link: &'a list::SingleListLink<'s, Self>) -> &'a Self
    where
        Self: Sized
    {
        unsafe {
            Self::from_bit_trie_link(mem::transmute::<&list::SingleListLink<'s, Self>, &BitTrieLink<'s, NCPL, Self>>(link))
        }
    }

    fn data(&self) -> usize;
    fn bit_trie_link(&self) -> &BitTrieLink<'s, NCPL, Self>
    where
        Self: Sized;
    fn from_bit_trie_link<'a>(link: &'a BitTrieLink<'s, NCPL, Self>) -> &'a Self
    where
        Self: Sized;
}

const BIT_TRIE_LINK_OFFSET: usize = 4; /*mem::offset_of!(T, list_link)*/

impl<'s, const NCPL: usize, T: 's> BitTrieRoot<'s, NCPL, T>
where T: BitTrieLinkAdapter<'s, NCPL> {
    pub fn unlink_lowest(&self) -> Option<&'s T> {
        self.unlink_lowest_in_subtree(self.root.get(), 0, usize::MAX, None, 0)
    }

    pub fn unlink_eq_or_above(&self, data: usize) -> Option<&'s T> {
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
            let index = (data >> shift) & (NCPL - 1);
            ptr = unsafe { cur.children.get_unchecked(index).get() };
            for i in index + 1..NCPL {
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

    fn unlink_lowest_in_subtree(&self, ptr: Option<&'s BitTrieLink<'s, NCPL, T>>, parents_index: usize, nearest_above_data: usize, nearest_above: Option<&'s BitTrieLink<'s, NCPL, T>>, nearest_above_parents_index: usize) -> Option<&'s T> {
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
            ptr_index = (0..NCPL)
                .find_map(|i| unsafe { cur.children.get_unchecked(i).get() }.map(|link| (link, i)) );
        }
        nearest_above.map(|link| {
            self.unlink_chunk(link, nearest_above_parents_index);
            T::from_bit_trie_link(link)
        })
    }

    fn unlink_chunk(&self, link: &'s BitTrieLink<'s, NCPL, T>, parents_index: usize) {
        if let Some(next) = self.list_for_data(link).pop_front() {
            self.replace_chunk(link, next.bit_trie_link(), parents_index);
        } else {
            let mut ptr_index = (0..NCPL)
                .rev()
                .find_map(|i| unsafe { link.children.get_unchecked(i).get() }.map(|link| (link, i)));
            let mut last_ptr_index = ptr_index;
            while let Some((cur, index)) = ptr_index {
                last_ptr_index = ptr_index;
                ptr_index = (0..NCPL)
                    .rev()
                    .find_map(|i| unsafe { cur.children.get_unchecked(i).get() }.map(|link| (link, i)));
            }
            if let Some((last_link, last_parents_index)) = last_ptr_index {
                self.replace_chunk(link, last_link, last_parents_index);
            } else {
                if let Some(parent) = link.parent.get() {
                    unsafe { parent.children.get_unchecked(parents_index) }.set(None);
                } else {
                    self.root.set(None);
                }
                link.clear_trie_link();
            }
        }
    }

    fn replace_chunk(&self, cur: &'s BitTrieLink<'s, NCPL, T>, replaced: &'s BitTrieLink<'s, NCPL, T>, parents_index: usize) {
        replaced.parent.set(cur.parent.get());
        if let Some(parent) = cur.parent.get() {
            unsafe { parent.children.get_unchecked(parents_index) }.set(Some(replaced));
        } else {
            self.root.set(Some(replaced));
        }
        for i in 0..NCPL {
            unsafe {
                replaced.children.get_unchecked(i).set(cur.children.get_unchecked(i).get());
                cur.children.get_unchecked(i).get().map(|child| child.parent.set(Some(replaced)));
            }
        }
        cur.clear_trie_link();
    }

    fn list_for_data(&self, link: &'s BitTrieLink<'s, NCPL, T>) -> list::LinkedStack<'_, 's, T, ChainTag> {
        list::LinkedStack::new(&link.chain)
    }

    pub fn insert(&self, chunk: &'s T) {
        let mut shift = self.shift;
        let mut ptr = &self.root;
        let mut parent = None;
        while let Some(cur) = ptr.get() {
            let cur_chunk = T::from_bit_trie_link(cur);
            if cur_chunk.data() == chunk.data() {
                self.list_for_data(cur).push_front(chunk);
                return;
            }
            let index = (chunk.data() >> shift) & (NCPL - 1);
            ptr = unsafe { &cur.children.get_unchecked(index) };
            parent = Some(cur);
            shift -= 2;
        }
        ptr.set(Some(chunk.bit_trie_link()));
        chunk.bit_trie_link().parent.set(parent);
    }
}
