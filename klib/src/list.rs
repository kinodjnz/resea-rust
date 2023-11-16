use core::cell::Cell;
use core::marker::PhantomData;

pub struct ListLink<T: 'static> {
    next: Cell<Option<&'static ListLink<T>>>,
    prev: Cell<Option<&'static ListLink<T>>>,
}

pub struct RemovableStartLink<T: 'static> {
    next: Cell<Option<&'static ListLink<T>>>,
}

pub struct SingleListLink<T: 'static> {
    next: Cell<Option<&'static SingleListLink<T>>>,
}

impl<T: 'static> SingleListLink<T> {
    pub fn set_next(&self, link: Option<&'static SingleListLink<T>>) {
        self.next.set(link)
    }

    pub fn next(&self) -> Option<&'static SingleListLink<T>> {
        self.next.get()
    }
}

pub trait StartLink<T: 'static, Link> {
    fn set_next(&self, link: Option<&'static Link>);
    fn set_prev(&self, link: Option<&'static Link>);
    fn next(&self) -> Option<&'static Link>;
}

impl<T: 'static> ListLink<T> {
    pub fn reset(&self) {
        self.next.set(None);
        self.prev.set(None);
    }
}

impl<T: 'static> SingleListLink<T> {
    pub const fn zeroed() -> Self {
        SingleListLink { next: Cell::new(None) }
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

pub trait SingleLinkAdapter<LinkTag> {
    fn link(&self) -> &SingleListLink<Self>
    where
        Self: Sized;
    fn from_link(link: &SingleListLink<Self>) -> &Self
    where
        Self: Sized;
}

pub struct LinkedList<'a, T: 'static, LinkTag> {
    link_start: &'a ListLink<T>,
    phantom_tag: PhantomData<LinkTag>,
}

pub struct RemovableLinkedStack<'a, T: 'static, LinkTag> {
    link_start: &'a RemovableStartLink<T>,
    phantom_tag: PhantomData<LinkTag>,
}

pub struct LinkedStack<'a, T: 'static, LinkTag> {
    link_start: &'a SingleListLink<T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, T: 'static> StartLink<T, ListLink<T>> for ListLink<T> {
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

impl<'a, T: 'static> StartLink<T, ListLink<T>> for RemovableStartLink<T> {
    fn set_next(&self, link: Option<&'static ListLink<T>>) {
        self.next.set(link);
    }

    fn set_prev(&self, _link: Option<&'static ListLink<T>>) {
    }

    fn next(&self) -> Option<&'static ListLink<T>> {
        self.next.get()
    }
}

pub trait RemovableLinkedStackOps<'a, T: 'static, LinkTag, Start>
where
    T: LinkAdapter<LinkTag>,
    Start: 'a + StartLink<T, ListLink<T>>,
{
    fn link_start(&self) -> &'a Start;

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

impl<'a, T: 'static, LinkTag: 'a>
    RemovableLinkedStackOps<'a, T, LinkTag, ListLink<T>>
for LinkedList<'a, T, LinkTag>
where
    T: LinkAdapter<LinkTag>,
{
    fn link_start(&self) -> &'a ListLink<T> {
        self.link_start
    }
}

impl<'a, T: 'static, LinkTag: 'a>
    RemovableLinkedStackOps<'a, T, LinkTag, RemovableStartLink<T>>
for RemovableLinkedStack<'a, T, LinkTag>
where
    T: LinkAdapter<LinkTag>,
{
    fn link_start(&self) -> &'a RemovableStartLink<T> {
        self.link_start
    }
}

impl<'a, T: 'static, LinkTag: 'a> RemovableLinkedStack<'a, T, LinkTag>
where
    T: LinkAdapter<LinkTag>,
{
    pub fn new(link_start: &'a RemovableStartLink<T>) -> Self {
        RemovableLinkedStack {
            link_start,
            phantom_tag: PhantomData,
        }
    }
}

impl<'a, T: 'static, LinkTag> LinkedList<'a, T, LinkTag>
  where
    T: LinkAdapter<LinkTag>,
{
    pub fn new(link_start: &'a ListLink<T>) -> Self {
        LinkedList {
            link_start,
            phantom_tag: PhantomData,
        }
    }

    // fn link_start(&self) -> &'a ListLink<T> {
    //     self.link_start
    // }

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

impl<'a, T: 'static, LinkTag> LinkedStack<'a, T, LinkTag>
  where
    T: SingleLinkAdapter<LinkTag>,
{
    pub fn new(link_start: &'a SingleListLink<T>) -> Self {
        LinkedStack {
            link_start,
            phantom_tag: PhantomData,
        }
    }

    fn link_start(&self) -> &'a SingleListLink<T> {
        self.link_start
    }

    pub fn push_front(&mut self, elem: &'static T) {
        elem.link().next.set(self.link_start().next());
        self.link_start().set_next(Some(elem.link()));
    }

    pub fn pop_front(&mut self) -> Option<&'static T> {
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
