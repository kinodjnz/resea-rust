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

impl<T> CellListLinkOps<T> for UnsafeCell<ListLink<T>> {
    fn next(&self) -> Option<&'static UnsafeCell<T>> {
        unsafe { (&*self.get()).next }
    }
    fn prev(&self) -> Option<&'static UnsafeCell<T>> {
        unsafe { (&*self.get()).prev }
    }
}

pub trait LinkAdapter<T, LinkTag> {
    fn link(&self) -> &ListLink<T>;
    fn link_mut(&mut self) -> &mut ListLink<T>;
}

pub struct LinkedList<'a, C: ContainerAdapter<T>, T: 'static, LinkTag> {
    container: &'a C,
    link_start: &'a mut UnsafeCell<ListLink<T>>,
    phantom_t: PhantomData<T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, C: ContainerAdapter<T>, T: LinkAdapter<T, LinkTag>, LinkTag: 'a>
    LinkedList<'a, C, T, LinkTag>
{
    pub fn new(container: &'a C, link_start: &'a mut UnsafeCell<ListLink<T>>) -> Self {
        LinkedList {
            container,
            link_start,
            phantom_t: PhantomData,
            phantom_tag: PhantomData,
        }
    }

    pub fn push_back(&mut self, elem: &'static UnsafeCell<T>) {
        if let Some(prev) = self.link_start.prev() {
            self.container.map_mut2(elem, prev, |elem_mut, prev| {
                let prev_link = prev.link_mut();
                let elem_link = elem_mut.link_mut();
                elem_link.set_next(prev_link.next);
                elem_link.set_prev(self.link_start.prev());
                prev_link.set_next(Some(elem));
                self.link_start.get_mut().set_prev(Some(elem));
            });
        } else {
            self.container.map_mut1(elem, |elem| {
                let elem_link = elem.link_mut();
                elem_link.set_next(self.link_start.next());
                elem_link.set_prev(self.link_start.prev());
            });
            self.link_start.get_mut().set_next(Some(elem));
            self.link_start.get_mut().set_prev(Some(elem));
        }
    }

    pub fn pop_front(&mut self) -> Option<&'static UnsafeCell<T>> {
        if let Some(front) = self.link_start.next() {
            if let Some(next) = unsafe { &*front.get() }.link().next {
                self.container.map_mut2(front, next, |front, next_mut| {
                    let next_link = next_mut.link_mut();
                    next_link.set_prev(None);
                    self.link_start.get_mut().set_next(Some(next));
                    let front_link = front.link_mut();
                    front_link.set_next(None);
                    front_link.set_prev(None);
                });
            } else {
                self.link_start.get_mut().set_next(None);
                self.link_start.get_mut().set_prev(None);
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

    /*
    fn remove(&mut self, elem: &mut T) {
        let elem = (self.element_adapter)(elem).element();
        let next: &mut ListLink<PointerType> = if elem.next() == CONTAINER {
            self.element
        } else {
            (self.element_adapter)(self.container.deref_pointer_mut(elem.next())).element_mut()
        };
        let prev: &mut ListLink<PointerType> = if elem.prev() == CONTAINER {
            self.element
        } else {
            (self.element_adapter)(self.container.deref_pointer_mut(elem.prev())).element_mut()
        };
        prev.set_next(elem.next());
        next.set_prev(elem.prev());
    }
    */
}
