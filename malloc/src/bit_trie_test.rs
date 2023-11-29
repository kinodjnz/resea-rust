use crate::bit_trie::*;
use alloc::string::String;
use core::cell::Cell;
use core::mem;
use klib::list;

#[derive(Debug, Eq, PartialEq)]
struct Chunk<'s> {
    value: usize,
    link: BitTrieLink<'s, 4, Chunk<'s>>,
}

impl<'s> Chunk<'s> {
    fn new(value: usize) -> Self {
        Self {
            value: value,
            link: BitTrieLink::init([
                Cell::new(Option::None),
                Cell::new(Option::None),
                Cell::new(Option::None),
                Cell::new(Option::None),
            ]),
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
                mem::transmute::<&BitTrieLink<'s, 4, Self>, usize>(link)
                    - mem::offset_of!(Chunk, link),
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

#[test]
fn bit_trie_insert_chain() {
    let trie = BitTrieRoot::<4, Chunk>::new(8);
    let chunk_a = Chunk::new(3);
    let chunk_b = Chunk::new(3);

    trie.insert(&chunk_a);
    trie.insert(&chunk_b);
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_b)));
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_a)));
    assert_eq!(trie.unlink_lowest(), None);
}

#[test]
fn bit_trie_insert_trie() {
    let trie = BitTrieRoot::<4, Chunk>::new(8);
    let chunk_a = Chunk::new(3);
    let chunk_b = Chunk::new(4);

    trie.insert(&chunk_a);
    trie.insert(&chunk_b);
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_a)));
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_b)));
    assert_eq!(trie.unlink_lowest(), None);
}

#[test]
fn bit_trie_insert_3() {
    let trie = BitTrieRoot::<4, Chunk>::new(4);
    let chunk_a = Chunk::new(1);
    let chunk_b = Chunk::new(2);
    let chunk_c = Chunk::new(3);
    let chunk_d = Chunk::new(2);
    let chunk_e = Chunk::new(4);

    trie.insert(&chunk_a);
    trie.insert(&chunk_b);
    trie.insert(&chunk_c);
    trie.insert(&chunk_d);
    trie.insert(&chunk_e);
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_a)));
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_d)));
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_b)));
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_c)));
    assert_eq!(address_of(trie.unlink_lowest()), address_of(Some(&chunk_e)));
}

#[test]
fn bit_trie_unlink_eq_or_above() {
    let trie = BitTrieRoot::<4, Chunk>::new(4);
    let chunk_a = Chunk::new(2);
    let chunk_b = Chunk::new(3);

    trie.insert(&chunk_a);
    trie.insert(&chunk_b);
    assert_eq!(address_of(trie.unlink_eq_or_above(4)), None);
    assert_eq!(
        address_of(trie.unlink_eq_or_above(3)),
        address_of(Some(&chunk_b))
    );
    assert_eq!(
        address_of(trie.unlink_eq_or_above(2)),
        address_of(Some(&chunk_a))
    );
    assert_eq!(trie.unlink_lowest(), None);
}

#[test]
fn bit_trie_unlink_eq_or_above_2() {
    let trie = BitTrieRoot::<4, Chunk>::new(4);
    let chunk_a = Chunk::new(5);
    let chunk_b = Chunk::new(6);
    let chunk_c = Chunk::new(2);
    let chunk_d = Chunk::new(4);
    let chunk_e = Chunk::new(1);

    trie.insert(&chunk_a);
    trie.insert(&chunk_b);
    trie.insert(&chunk_c);
    trie.insert(&chunk_d);
    trie.insert(&chunk_e);
    assert_eq!(
        address_of(trie.unlink_eq_or_above(3)),
        address_of(Some(&chunk_d))
    );
    assert_eq!(
        address_of(trie.unlink_eq_or_above(3)),
        address_of(Some(&chunk_a))
    );
    assert_eq!(
        address_of(trie.unlink_eq_or_above(3)),
        address_of(Some(&chunk_b))
    );
    assert_eq!(trie.unlink_eq_or_above(3), None);
    assert_eq!(
        address_of(trie.unlink_eq_or_above(1)),
        address_of(Some(&chunk_e))
    );
    assert_eq!(
        address_of(trie.unlink_eq_or_above(1)),
        address_of(Some(&chunk_c))
    );
    assert_eq!(trie.unlink_eq_or_above(1), None);
}

fn address_of<T>(x: Option<&T>) -> Option<*const T> {
    x.map(|x| x as *const _)
}

#[allow(dead_code)]
fn dump_trie<'s>(trie: &BitTrieRoot<'s, 4, Chunk<'s>>) {
    let mut s = String::new();
    trie.write(&mut s);
    assert!(false, "{}", s);
}
