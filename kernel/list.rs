use core::marker::PhantomData;

pub trait ContainerAdapter<'t, T> {
    fn as_mut1(&mut self, t: &'t T) -> &'t mut T;
    fn as_mut2(&mut self, t1: &'t T, t2: &'t T) -> (&mut T, &mut T);
}

pub struct ListLink<'t, T> {
    next: Option<&'t T>,
    prev: Option<&'t T>,
}

impl<'t, T> ListLink<'t, T> {
    pub fn new() -> ListLink<'t, T> {
        ListLink {
            next: None,
            prev: None,
        }
    }
}

impl<'t, T> ListLink<'t, T> {
    pub fn next(&self) -> Option<&'t T> {
        self.next
    }

    pub fn prev(&self) -> Option<&'t T> {
        self.prev
    }

    pub fn set_next<'s>(&mut self, p: Option<&'s T>) {
        self.next = p.map(|x| unsafe { &*(x as *const T) });
    }

    pub fn set_prev<'s>(&mut self, p: Option<&'s T>) {
        self.prev = p.map(|x| unsafe { &*(x as *const T) });
    }
}

pub trait LinkAdapter<'t, T, LinkTag> {
    fn link(&'t self) -> &'t ListLink<'t, T>;
    fn link_mut(&mut self) -> &mut ListLink<'t, T>;
}

pub struct LinkedList<'a, 't, C: ContainerAdapter<'t, T>, T, LinkTag> {
    container: &'a mut C,
    link_start: &'a mut ListLink<'t, T>,
    phantom_t: PhantomData<T>,
    phantom_tag: PhantomData<LinkTag>,
}

impl<'a, 't, C: ContainerAdapter<'t, T>, T: LinkAdapter<'t, T, LinkTag>, LinkTag: 'a>
    LinkedList<'a, 't, C, T, LinkTag>
{
    pub fn new(container: &'a mut C, link_start: &'a mut ListLink<'t, T>) -> Self {
        LinkedList {
            container,
            link_start,
            phantom_t: PhantomData,
            phantom_tag: PhantomData,
        }
    }

    pub fn push_back<'s>(&'s mut self, elem: &'t T) {
        if let Some(prev) = self.link_start.prev() {
            let (elem, prev) = self.container.as_mut2(elem, prev);
            let prev_link = prev.link_mut();
            let elem_link = elem.link_mut();
            elem_link.set_next(prev_link.next());
            elem_link.set_prev(self.link_start.prev());
            prev_link.set_next(Some(elem));
            self.link_start.set_prev(Some(elem));
        } else {
            let elem = self.container.as_mut1(elem);
            let elem_link = elem.link_mut();
            elem_link.set_next(self.link_start.next());
            elem_link.set_prev(self.link_start.prev());
            self.link_start.set_next(Some(elem));
            self.link_start.set_prev(Some(elem));
        }
    }

    pub fn pop_front(&mut self) -> Option<&'t T> {
        if let Some(front) = self.link_start.next() {
            if let Some(next) = front.link().next() {
                let (front, next) = self.container.as_mut2(front, next);
                let next_link = next.link_mut();
                next_link.set_prev(None);
                self.link_start.set_next(Some(next));
                let front_link = front.link_mut();
                front_link.set_next(None);
                front_link.set_prev(None);
            } else {
                self.link_start.set_next(None);
                self.link_start.set_prev(None);
                let front = self.container.as_mut1(front);
                let front_link = front.link_mut();
                front_link.set_next(None);
                front_link.set_prev(None);
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
