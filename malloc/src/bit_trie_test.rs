use crate::bit_trie::*;
use core::mem;
use core::cell::Cell;
use klib::list;

#[derive(Debug, Eq, PartialEq)]
struct Chunk<'s> {
    value: usize,
    link: BitTrieLink<'s, 4, Chunk<'s>>,
}

impl<'s> Chunk<'s> {
    const fn zeroed() -> Self {
        Self {
            value: 3,
            link: BitTrieLink::init([Cell::new(Option::None), Cell::new(Option::None), Cell::new(Option::None), Cell::new(Option::None)]),
        }
    }
}

impl<'s> BitTrieLinkAdapter<'s, 4> for Chunk<'s> {
    fn data(&self) -> usize {
        self.value
    }

    fn from_bit_trie_link<'a>(link: &'a BitTrieLink<'s, 4, Self>) -> &'a Self {
        unsafe {
            mem::transmute::<usize, &Self>(
                mem::transmute::<&BitTrieLink<'s, 4, Self>, usize>(link) - mem::offset_of!(Chunk, link),
            )
        }
    }

    fn bit_trie_link(&self) -> &BitTrieLink<'s, 4, Self> {
        &self.link
    }
}

impl<'s> list::SingleLinkAdapter<'s, ChainTag> for Chunk<'s> {
    fn link(&self) -> &list::SingleListLink<'s, Self> {
        BitTrieLinkAdapter::link(self)
    }

    fn from_link<'a>(link: &'a list::SingleListLink<'s, Self>) -> &'a Self {
        BitTrieLinkAdapter::from_link(link)
    }
}

// static mut CHUNK: Chunk = Chunk::zeroed();

#[test]
fn bit_trie_insert() {
    let trie = BitTrieRoot::<4, Chunk>::new(8);
    let chunk = Chunk::zeroed();

    trie.insert(&chunk);
    assert_eq!(trie.unlink_lowest(), Option::Some(&chunk));
    assert_eq!(trie.unlink_lowest(), Option::None);
}
