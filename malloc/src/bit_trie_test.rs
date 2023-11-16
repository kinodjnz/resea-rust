use crate::bit_trie::*;
use core::mem;
use core::cell::Cell;
use klib::list;

struct Chunk {
    value: usize,
    link: BitTrieLink<2, Chunk>,
}

impl Chunk {
    const fn zeroed() -> Self {
        Self {
            value: 3,
            link: BitTrieLink::init([Cell::new(Option::None), Cell::new(Option::None)]),
        }
    }
}

impl BitTrieLinkAdapter<2> for Chunk {
    fn data(&self) -> usize {
        self.value
    }

    fn from_bit_trie_link(link: &BitTrieLink<2, Self>) -> &Self {
        unsafe {
            mem::transmute::<usize, &Self>(
                mem::transmute::<&BitTrieLink<2, Self>, usize>(link) - mem::offset_of!(Chunk, link),
            )
        }
    }

    fn bit_trie_link(&self) -> &BitTrieLink<2, Self> {
        &self.link
    }
}

impl list::SingleLinkAdapter<ChainTag> for Chunk {
    fn link(&self) -> &list::SingleListLink<Self> {
        BitTrieLinkAdapter::link(self)
    }

    fn from_link(link: &list::SingleListLink<Self>) -> &Self {
        BitTrieLinkAdapter::from_link(link)
    }
}

static mut CHUNK: Chunk = Chunk::zeroed();

#[test]
fn bit_trie_insert() {
    let trie = BitTrieRoot::<2, Chunk>::new(8);
    // let chunk = Chunk { value: 3, link: unsafe { mem::zeroed() } };

    trie.insert(unsafe { &CHUNK });
    assert_eq!(2, 3);
}
