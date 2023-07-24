use core::cell::Cell;
use core::marker::PhantomData;

pub mod ops {
    pub use crate::list::LinkedStackOps;
}

pub struct ListLink<T: 'static> {
    next: Cell<Option<&'static ListLink<T>>>,
    prev: Cell<Option<&'static ListLink<T>>>,
}

pub struct SingleListLink<T: 'static> {
    next: Cell<Option<&'static ListLink<T>>>,
}

pub trait StartLink<T: 'static> {
    fn set_next(&self, link: Option<&'static ListLink<T>>);
    fn set_prev(&self, link: Option<&'static ListLink<T>>);
    fn next(&self) -> Option<&'static ListLink<T>>;
}

impl<T: 'static> StartLink<T> for ListLink<T> {
    fn set_next(&self, link: Option<&'static ListLink<T>>) {
        self.next.set(link);
    }

    fn set_prev(&self, link: Option<&'static ListLink<T>>) {
        self.prev.set(link);
    }

    fn next(&self) -> Option<&'static ListLink<T>> {
        self.next.get()
    }
}

impl<T: 'static> StartLink<T> for SingleListLink<T> {
    fn set_next(&self, link: Option<&'static ListLink<T>>) {
        self.next.set(link);
    }

    fn set_prev(&self, _link: Option<&'static ListLink<T>>) {}

    fn next(&self) -> Option<&'static ListLink<T>> {
        self.next.get()
    }
}

impl<T: 'static> ListLink<T> {
    pub fn reset(&self) {
        self.next.set(None);
        self.prev.set(None);
    }
}

pub trait LinkAdapter<LinkTag> {
    fn link(&self) -> &ListLink<Self>
    where
        Self: Sized;
    fn from_link(link: &ListLink<Self>) -> &Self
    where
        Self: Sized;
}

pub struct LinkedList<'a, T: 'static, LinkTag> {
    link_start: &'a ListLink<T>,
    phantom_tag: PhantomData<LinkTag>,
}

pub struct LinkedStack<'a, T: 'static, LinkTag> {
    link_start: &'a SingleListLink<T>,
    phantom_tag: PhantomData<LinkTag>,
}

pub trait LinkedStackOps<'a, T: 'static, LinkTag, S: 'a>
where
    T: LinkAdapter<LinkTag>,
    S: StartLink<T>,
{
    fn link_start(&self) -> &'a S;

    fn push_front(&mut self, elem: &'static T) {
        if let Some(next) = self.link_start().next() {
            elem.link().next.set(Some(next));
            elem.link().prev.set(None);
            next.prev.set(Some(elem.link()));
            self.link_start().set_next(Some(elem.link()));
        } else {
            elem.link().next.set(None);
            elem.link().prev.set(None);
            self.link_start().set_next(Some(elem.link()));
            self.link_start().set_prev(Some(elem.link()));
        }
    }

    fn pop_front(&mut self) -> Option<&'static T> {
        if let Some(front) = self.link_start().next() {
            if let Some(next) = front.next.get() {
                next.prev.set(None);
                self.link_start().set_next(Some(next));
                front.next.set(None);
                front.prev.set(None);
            } else {
                self.link_start().set_next(None);
                self.link_start().set_prev(None);
                front.next.set(None);
                front.prev.set(None);
            }
            Some(T::from_link(front))
        } else {
            // list is empty
            None
        }
    }

    fn remove(&mut self, elem: &'static T) {
        if let Some(next) = elem.link().next.get() {
            next.prev.set(elem.link().prev.get());
        } else {
            self.link_start().set_prev(elem.link().prev.get());
        }
        if let Some(prev) = elem.link().prev.get() {
            prev.next.set(elem.link().next.get());
        } else {
            self.link_start().set_next(elem.link().next.get());
        }
        elem.link().next.set(None);
        elem.link().prev.set(None);
    }

    fn empty(&self) -> bool {
        self.link_start().next().is_none()
    }
}

impl<'a, T: LinkAdapter<LinkTag>, LinkTag: 'a> LinkedStackOps<'a, T, LinkTag, ListLink<T>>
    for LinkedList<'a, T, LinkTag>
{
    fn link_start(&self) -> &'a ListLink<T> {
        self.link_start
    }
}

impl<'a, T: LinkAdapter<LinkTag>, LinkTag: 'a> LinkedStackOps<'a, T, LinkTag, SingleListLink<T>>
    for LinkedStack<'a, T, LinkTag>
{
    fn link_start(&self) -> &'a SingleListLink<T> {
        self.link_start
    }
}

impl<'a, T: LinkAdapter<LinkTag>, LinkTag: 'a> LinkedStack<'a, T, LinkTag> {
    pub fn new(link_start: &'a SingleListLink<T>) -> Self {
        LinkedStack {
            link_start,
            phantom_tag: PhantomData,
        }
    }
}

impl<'a, T: LinkAdapter<LinkTag>, LinkTag: 'a> LinkedList<'a, T, LinkTag> {
    pub fn new(link_start: &'a ListLink<T>) -> Self {
        LinkedList {
            link_start,
            phantom_tag: PhantomData,
        }
    }

    pub fn push_back(&mut self, elem: &'static T) {
        if let Some(prev) = self.link_start.prev.get() {
            elem.link().next.set(None);
            elem.link().prev.set(Some(prev));
            prev.next.set(Some(elem.link()));
            self.link_start.prev.set(Some(elem.link()));
        } else {
            elem.link().next.set(None);
            elem.link().prev.set(None);
            self.link_start.next.set(Some(elem.link()));
            self.link_start.prev.set(Some(elem.link()));
        }
    }

    pub fn iter(&self) -> ListIterator<'_, T, LinkTag> {
        ListIterator {
            current: self.link_start,
            phantom_tag: PhantomData,
        }
    }
}

pub struct ListIterator<'a, T: LinkAdapter<LinkTag> + 'static, LinkTag> {
    current: &'a ListLink<T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, T: LinkAdapter<LinkTag> + 'static, LinkTag> Iterator for ListIterator<'a, T, LinkTag> {
    type Item = &'static T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current.next.get();
        if let Some(next) = next {
            self.current = next;
            Some(T::from_link(next))
        } else {
            None
        }
    }
}
