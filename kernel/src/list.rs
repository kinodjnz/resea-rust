use core::cell::Cell;
use core::marker::PhantomData;

pub struct ListLink<T: 'static> {
    next: Cell<Option<&'static T>>,
    prev: Cell<Option<&'static T>>,
}

impl<T: 'static> ListLink<T> {
    pub fn reset(&self) {
        self.next.set(None);
        self.prev.set(None);
    }
}

pub trait LinkAdapter<T, LinkTag> {
    fn link(&self) -> &ListLink<T>;
}

pub struct LinkedList<'a, T: 'static, LinkTag> {
    link_start: &'a ListLink<T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, T: LinkAdapter<T, LinkTag>, LinkTag: 'a> LinkedList<'a, T, LinkTag> {
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
            prev.link().next.set(Some(elem));
            self.link_start.prev.set(Some(elem));
        } else {
            elem.link().next.set(None);
            elem.link().prev.set(None);
            self.link_start.next.set(Some(elem));
            self.link_start.prev.set(Some(elem));
        }
    }

    pub fn pop_front(&mut self) -> Option<&'static T> {
        if let Some(front) = self.link_start.next.get() {
            if let Some(next) = front.link().next.get() {
                next.link().prev.set(None);
                self.link_start.next.set(Some(next));
                front.link().next.set(None);
                front.link().prev.set(None);
            } else {
                self.link_start.next.set(None);
                self.link_start.prev.set(None);
                front.link().next.set(None);
                front.link().prev.set(None);
            }
            Some(front)
        } else {
            // list is empty
            None
        }
    }

    pub fn remove(&mut self, elem: &'static T) {
        if let Some(next) = elem.link().next.get() {
            next.link().prev.set(elem.link().prev.get());
        } else {
            self.link_start.prev.set(elem.link().prev.get());
        }
        if let Some(prev) = elem.link().prev.get() {
            prev.link().next.set(elem.link().next.get());
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

pub struct ListIterator<'a, T: LinkAdapter<T, LinkTag> + 'static, LinkTag> {
    current: &'a ListLink<T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, T: LinkAdapter<T, LinkTag> + 'static, LinkTag> Iterator for ListIterator<'a, T, LinkTag> {
    type Item = &'static T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current.next.get();
        if let Some(next) = next {
            self.current = next.link();
        }
        next
    }
}
