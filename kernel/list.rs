use core::cell::UnsafeCell;
use core::marker::PhantomData;

pub trait ContainerAdapter<T> {
    fn map_mut1<R, F: FnOnce(&mut T) -> R>(&self, t1: &'static UnsafeCell<T>, f: F) -> R;
    fn map_mut2<R, F: FnOnce(&mut T, &mut T) -> R>(
        &self,
        t1: &'static UnsafeCell<T>,
        t2: &'static UnsafeCell<T>,
        f: F,
    ) -> R;
}

pub struct ListLink<T: 'static> {
    next: Option<&'static UnsafeCell<T>>,
    prev: Option<&'static UnsafeCell<T>>,
}

impl<T: 'static> ListLink<T> {
    pub fn new() -> ListLink<T> {
        ListLink {
            next: None,
            prev: None,
        }
    }
}

impl<T> ListLink<T> {
    pub fn set_next(&mut self, p: Option<&'static UnsafeCell<T>>) {
        self.next = p;
    }

    pub fn set_prev(&mut self, p: Option<&'static UnsafeCell<T>>) {
        self.prev = p;
    }
}

pub trait CellListLinkOps<T> {
    fn next(&self) -> Option<&'static UnsafeCell<T>>;
    fn prev(&self) -> Option<&'static UnsafeCell<T>>;
}

/*
impl<T> CellListLinkOps<T> for UnsafeCell<ListLink<T>> {
    fn next(&self) -> Option<&'static UnsafeCell<T>> {
        unsafe { (&*self.get()).next }
    }
    fn prev(&self) -> Option<&'static UnsafeCell<T>> {
        unsafe { (&*self.get()).prev }
    }
}
*/

pub trait LinkAdapter<T, LinkTag> {
    fn link(&self) -> &ListLink<T>;
    fn link_mut(&mut self) -> &mut ListLink<T>;
}

impl<'a, T: LinkAdapter<T, LinkTag>, LinkTag: 'a> LinkAdapter<T, LinkTag> for UnsafeCell<T> {
    fn link(&self) -> &ListLink<T> {
        unsafe { &*self.get() }.link()
    }
    fn link_mut(&mut self) -> &mut ListLink<T> {
        self.get_mut().link_mut()
    }
}

pub struct LinkedList<'a, C: ContainerAdapter<T>, T: 'static, LinkTag> {
    container: &'a C,
    link_start: &'a mut ListLink<T>,
    phantom_t: PhantomData<T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, C: ContainerAdapter<T>, T: LinkAdapter<T, LinkTag>, LinkTag: 'a>
    LinkedList<'a, C, T, LinkTag>
{
    pub fn new(container: &'a C, link_start: &'a mut ListLink<T>) -> Self {
        LinkedList {
            container,
            link_start,
            phantom_t: PhantomData,
            phantom_tag: PhantomData,
        }
    }

    pub fn push_back(&mut self, elem: &'static UnsafeCell<T>) {
        if let Some(prev) = self.link_start.prev {
            self.container.map_mut2(elem, prev, |elem_mut, prev_mut| {
                let prev_link = prev_mut.link_mut();
                let elem_link = elem_mut.link_mut();
                elem_link.set_next(None);
                elem_link.set_prev(Some(prev));
                prev_link.set_next(Some(elem));
                self.link_start.set_prev(Some(elem));
            });
        } else {
            self.container.map_mut1(elem, |elem| {
                let elem_link = elem.link_mut();
                elem_link.set_next(None);
                elem_link.set_prev(None);
            });
            self.link_start.set_next(Some(elem));
            self.link_start.set_prev(Some(elem));
        }
    }

    pub fn pop_front(&mut self) -> Option<&'static UnsafeCell<T>> {
        if let Some(front) = self.link_start.next {
            if let Some(next) = unsafe { &*front.get() }.link().next {
                self.container.map_mut2(front, next, |front, next_mut| {
                    let next_link = next_mut.link_mut();
                    next_link.set_prev(None);
                    self.link_start.set_next(Some(next));
                    let front_link = front.link_mut();
                    front_link.set_next(None);
                    front_link.set_prev(None);
                });
            } else {
                self.link_start.set_next(None);
                self.link_start.set_prev(None);
                self.container.map_mut1(front, |front| {
                    let front_link = front.link_mut();
                    front_link.set_next(None);
                    front_link.set_prev(None);
                });
            }
            Some(front)
        } else {
            // list is empty
            None
        }
    }

    pub fn remove(&mut self, elem: &'static UnsafeCell<T>) {
        let elem_link = unsafe { &*elem.get() }.link();
        if let Some(next) = elem_link.next {
            self.container.map_mut2(elem, next, |elem, next| {
                next.link_mut().set_prev(elem.link().prev);
            })
        } else {
            self.link_start.set_prev(elem_link.prev);
        }
        if let Some(prev) = elem_link.prev {
            self.container.map_mut2(elem, prev, |elem, prev| {
                prev.link_mut().set_next(elem.link().next);
            })
        } else {
            self.link_start.set_next(elem_link.next);
        }
        self.container.map_mut1(elem, |elem| {
            elem.link_mut().set_next(None);
            elem.link_mut().set_prev(None);
        });
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
    type Item = &'static UnsafeCell<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current.next;
        if let Some(next) = next {
            self.current = next.link();
        }
        next
    }
}
