use core::marker::PhantomData;

pub trait ContainerAdapter<PointerType: Copy + PartialEq, T> {
    fn deref_pointer<'t>(&self, p: PointerType) -> &'t T;
    fn deref_pointer_mut(&mut self, p: PointerType) -> &mut T;
    fn as_mut(&mut self, t: &T) -> &mut T;
    fn as_mut2(&mut self, t1: &T, t2: &T) -> (&mut T, &mut T);
}

pub struct ListElement<PointerType> {
    next: PointerType,
    prev: PointerType,
}

impl<PointerType: Copy + PartialEq> ListElement<PointerType> {
    pub fn init(default: PointerType) -> ListElement<PointerType> {
        ListElement {
            next: default,
            prev: default,
        }
    }
}

impl<PointerType: Copy + PartialEq> ListElement<PointerType> {
    pub fn next(&self) -> PointerType {
        self.next
    }

    pub fn prev(&self) -> PointerType {
        self.prev
    }

    pub fn set_next(&mut self, p: PointerType) {
        self.next = p;
    }

    pub fn set_prev(&mut self, p: PointerType) {
        self.prev = p;
    }
}

pub trait ElementAdapter<Tag, PointerType: Copy + PartialEq> {
    fn element(&self) -> &ListElement<PointerType>;
    fn element_mut(&mut self) -> &mut ListElement<PointerType>;
    fn self_pointer(&self) -> PointerType;
}

type PointerType = u32;

pub struct LinkedList<'a, T, C: ContainerAdapter<PointerType, T>, ElementTag, const CONTAINER: PointerType> {
    container: &'a mut C,
    element: &'a mut ListElement<PointerType>,
    element_adapter: fn(&T) -> &dyn ElementAdapter<ElementTag, PointerType>,
    element_adapter_mut: fn(&mut T) -> &mut dyn ElementAdapter<ElementTag, PointerType>,
    phantom: PhantomData<T>,
}

impl<'a, T, C: ContainerAdapter<PointerType, T>, ElementTag, const CONTAINER: PointerType> LinkedList<'a, T, C, ElementTag, CONTAINER> {
    pub fn new(container: &'a mut C, element: &'a mut ListElement<PointerType>, element_adapter: fn(&T) -> &dyn ElementAdapter<ElementTag, PointerType>, element_adapter_mut: fn(&mut T) -> &mut dyn ElementAdapter<ElementTag, PointerType>) -> Self {
        LinkedList {
            container,
            element,
            element_adapter,
            element_adapter_mut,
            phantom: PhantomData,
        }
    }

    pub fn push_back(&mut self, elem: &T) {
        if self.element.next() == CONTAINER {
            // list is empty
            let elem = self.container.as_mut(elem);
            let elem_adapter = (self.element_adapter_mut)(elem);
            let elem_pointer = elem_adapter.self_pointer();
            elem_adapter.element_mut().set_next(elem_pointer);
            elem_adapter.element_mut().set_prev(elem_pointer);
            self.element.set_next(elem_pointer);
            self.element.set_prev(elem_pointer);
        } else {
            let prev = self.container.deref_pointer(self.element.prev());
            let (elem, prev) = self.container.as_mut2(elem, &*prev);
            let elem_adapter = (self.element_adapter_mut)(elem);
            let elem_pointer = elem_adapter.self_pointer();
            let prev_adapter = (self.element_adapter_mut)(prev);
            elem_adapter.element_mut().set_next(prev_adapter.element().next());
            elem_adapter.element_mut().set_prev(self.element.prev());
            prev_adapter.element_mut().set_next(elem_pointer);
            self.element.set_prev(elem_pointer);
        }
    }

    pub fn pop_front(&mut self) -> Option<&mut T> {
        if self.element.next() == CONTAINER {
            // list is empty
            None
        } else {
            let front = self.container.deref_pointer(self.element.next());
            let front_adapter = (self.element_adapter)(&*front);
            let next = self.container.deref_pointer(front_adapter.element().next());
            let (front, next) = self.container.as_mut2(&*front, &*next);
            let next_adapter = (self.element_adapter_mut)(next);
            self.element.set_next(next_adapter.self_pointer());
            next_adapter.element_mut().set_prev(CONTAINER);
            Some(front)
        }
    }

    /*
    fn remove(&mut self, elem: &mut T) {
        let elem = (self.element_adapter)(elem).element();
        let next: &mut ListElement<PointerType> = if elem.next() == CONTAINER {
            self.element
        } else {
            (self.element_adapter)(self.container.deref_pointer_mut(elem.next())).element_mut()
        };
        let prev: &mut ListElement<PointerType> = if elem.prev() == CONTAINER {
            self.element
        } else {
            (self.element_adapter)(self.container.deref_pointer_mut(elem.prev())).element_mut()
        };
        prev.set_next(elem.next());
        next.set_prev(elem.prev());
    }
    */
}
