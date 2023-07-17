use core::cell::Cell;
use core::marker::PhantomData;

pub struct ListLink<T: 'static> {
    next: Cell<Option<&'static ListLink<T>>>,
    prev: Cell<Option<&'static ListLink<T>>>,
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

    pub fn pop_front(&mut self) -> Option<&'static T> {
        if let Some(front) = self.link_start.next.get() {
            if let Some(next) = front.next.get() {
                next.prev.set(None);
                self.link_start.next.set(Some(next));
                front.next.set(None);
                front.prev.set(None);
            } else {
                self.link_start.next.set(None);
                self.link_start.prev.set(None);
                front.next.set(None);
                front.prev.set(None);
            }
            Some(T::from_link(front))
        } else {
            // list is empty
            None
        }
    }

    pub fn remove(&mut self, elem: &'static T) {
        if let Some(next) = elem.link().next.get() {
            next.prev.set(elem.link().prev.get());
        } else {
            self.link_start.prev.set(elem.link().prev.get());
        }
        if let Some(prev) = elem.link().prev.get() {
            prev.next.set(elem.link().next.get());
        } else {
            self.link_start.next.set(elem.link().next.get());
        }
        elem.link().next.set(None);
        elem.link().prev.set(None);
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
