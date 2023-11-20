use core::cell::Cell;
use core::marker::PhantomData;

pub struct ListLink<'s, T: 's> {
    next: Cell<Option<&'s ListLink<'s, T>>>,
    prev: Cell<Option<&'s ListLink<'s, T>>>,
}

pub struct RemovableStartLink<'s, T: 's> {
    next: Cell<Option<&'s ListLink<'s, T>>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SingleListLink<'s, T: 's> {
    next: Cell<Option<&'s SingleListLink<'s, T>>>,
}

impl<'s, T: 's> SingleListLink<'s, T> {
    pub fn set_next(&self, link: Option<&'s SingleListLink<'s, T>>) {
        self.next.set(link)
    }

    pub fn next(&self) -> Option<&'s SingleListLink<'s, T>> {
        self.next.get()
    }
}

pub trait StartLink<'s, T: 's, Link> {
    fn set_next(&self, link: Option<&'s Link>);
    fn set_prev(&self, link: Option<&'s Link>);
    fn next(&self) -> Option<&'s Link>;
}

impl<'s, T: 's> ListLink<'s, T> {
    pub fn reset(&self) {
        self.next.set(None);
        self.prev.set(None);
    }
}

impl<'s, T: 's> SingleListLink<'s, T> {
    pub const fn zeroed() -> Self {
        SingleListLink { next: Cell::new(None) }
    }
}

pub trait LinkAdapter<'s, LinkTag> {
    fn link(&self) -> &ListLink<'s, Self>
    where
        Self: Sized;
    fn from_link<'a>(link: &'a ListLink<'s, Self>) -> &'a Self
    where
        Self: Sized;
}

pub trait SingleLinkAdapter<'s, LinkTag> {
    fn link(&self) -> &SingleListLink<'s, Self>
    where
        Self: Sized;
    fn from_link<'a>(link: &'a SingleListLink<'s, Self>) -> &'a Self
    where
        Self: Sized;
}

pub struct LinkedList<'a, 's, T: 's, LinkTag> {
    link_start: &'a ListLink<'s, T>,
    phantom_tag: PhantomData<LinkTag>,
}

pub struct RemovableLinkedStack<'a, 's, T: 's, LinkTag> {
    link_start: &'a RemovableStartLink<'s, T>,
    phantom_tag: PhantomData<LinkTag>,
}

pub struct LinkedStack<'a, 's, T: 's, LinkTag> {
    link_start: &'a SingleListLink<'s, T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, 's, T: 's> StartLink<'s, T, ListLink<'s, T>> for ListLink<'s, T> {
    fn set_next(&self, link: Option<&'s ListLink<'s, T>>) {
        self.next.set(link);
    }

    fn set_prev(&self, link: Option<&'s ListLink<'s, T>>) {
        self.prev.set(link);
    }

    fn next(&self) -> Option<&'s ListLink<'s, T>> {
        self.next.get()
    }
}

impl<'a, 's, T: 's> StartLink<'s, T, ListLink<'s, T>> for RemovableStartLink<'s, T> {
    fn set_next(&self, link: Option<&'s ListLink<'s, T>>) {
        self.next.set(link);
    }

    fn set_prev(&self, _link: Option<&'s ListLink<'s, T>>) {
    }

    fn next(&self) -> Option<&'s ListLink<'s, T>> {
        self.next.get()
    }
}

pub trait RemovableLinkedStackOps<'a, 's, T: 's, LinkTag, Start>
where
    T: LinkAdapter<'s, LinkTag>,
    Start: 'a + StartLink<'s, T, ListLink<'s, T>>,
{
    fn link_start(&self) -> &'a Start;

    fn push_front(&mut self, elem: &'s T) {
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

    fn pop_front(&mut self) -> Option<&'s T> {
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

    fn remove(&mut self, elem: &'s T) {
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

impl<'a, 's, T: 's, LinkTag: 'a>
    RemovableLinkedStackOps<'a, 's, T, LinkTag, ListLink<'s, T>>
for LinkedList<'a, 's, T, LinkTag>
where
    T: LinkAdapter<'s, LinkTag>,
{
    fn link_start(&self) -> &'a ListLink<'s, T> {
        self.link_start
    }
}

impl<'a, 's, T: 's, LinkTag: 'a>
    RemovableLinkedStackOps<'a, 's, T, LinkTag, RemovableStartLink<'s, T>>
for RemovableLinkedStack<'a, 's, T, LinkTag>
where
    T: LinkAdapter<'s, LinkTag>,
{
    fn link_start(&self) -> &'a RemovableStartLink<'s, T> {
        self.link_start
    }
}

impl<'a, 's, T: 's, LinkTag: 'a> RemovableLinkedStack<'a, 's, T, LinkTag>
where
    T: LinkAdapter<'s, LinkTag>,
{
    pub fn new(link_start: &'a RemovableStartLink<'s, T>) -> Self {
        RemovableLinkedStack {
            link_start,
            phantom_tag: PhantomData,
        }
    }
}

impl<'a, 's, T: 's, LinkTag> LinkedList<'a, 's, T, LinkTag>
  where
    T: LinkAdapter<'s, LinkTag>,
{
    pub fn new(link_start: &'a ListLink<'s, T>) -> Self {
        LinkedList {
            link_start,
            phantom_tag: PhantomData,
        }
    }

    // fn link_start(&self) -> &'a ListLink<T> {
    //     self.link_start
    // }

    pub fn push_back(&mut self, elem: &'s T) {
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

    pub fn iter(&self) -> ListIterator<'_, 's, T, LinkTag> {
        ListIterator {
            current: self.link_start,
            phantom_tag: PhantomData,
        }
    }
}

pub struct ListIterator<'a, 's, T: LinkAdapter<'s, LinkTag> + 's, LinkTag> {
    current: &'a ListLink<'s, T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, 's, T: LinkAdapter<'s, LinkTag> + 's, LinkTag> Iterator for ListIterator<'a, 's, T, LinkTag> {
    type Item = &'s T;

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

impl<'a, 's, T: 's, LinkTag> LinkedStack<'a, 's, T, LinkTag>
  where
    T: SingleLinkAdapter<'s, LinkTag>,
{
    pub fn new(link_start: &'a SingleListLink<'s, T>) -> Self {
        LinkedStack {
            link_start,
            phantom_tag: PhantomData,
        }
    }

    fn link_start(&self) -> &'a SingleListLink<'s, T> {
        self.link_start
    }

    pub fn push_front(&mut self, elem: &'s T) {
        elem.link().next.set(self.link_start().next());
        self.link_start().set_next(Some(elem.link()));
    }

    pub fn pop_front(&mut self) -> Option<&'s T> {
        if let Some(front) = self.link_start().next() {
            self.link_start().set_next(front.next.get());
            front.next.set(None);
            Some(T::from_link(front))
        } else {
            // list is empty
            None
        }
    }

    pub fn empty(&self) -> bool {
        self.link_start().next().is_none()
    }
}
