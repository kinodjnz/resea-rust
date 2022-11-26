use core::convert;
use core::mem::transmute_copy;
use core::ops::{self, ControlFlow};
use crate::macros::*;

#[repr(align(4))]
#[allow(dead_code)]
pub enum KResult<T> {
    Ok(T),
    NoMemory,      // 1
    NotPermitted,  // 2
    WouldBlock,    // 3
    Aborted,       // 4
    TooLarge,      // 5
    TooSmall,      // 6
    NotFound,      // 7
    InvalidArg,    // 8
    InvalidTask,   // 9
    AlreadyExists, // 10
    Unavailable,   // 11
    NotAcceptable, // 12
    Empty,         // 13
    DontReply,     // 14
    InUse,         // 15
    TryAgain,      // 16
    NotReady,      // 17
}

pub struct IntoIter<T> {
    inner: Option<T>,
}

impl<T> KResult<T> {
    pub const fn is_ok(&self) -> bool {
        match self {
            KResult::Ok(_) => true,
            _ => false,
        }
    }

    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn ok(self) -> Option<T> {
        match self {
            KResult::Ok(x) => Some(x),
            _ => None,
        }
    }

    pub fn err_as_u32(&self) -> u32 {
        unsafe { transmute_copy(self) }
    }

    pub fn err_from_u32(e: u32) -> Self {
        if e == 0 {
            kpanic!(b"err_from_u32 called for ok");
        }
        unsafe { transmute_copy(&e) }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> KResult<U> {
        match self {
            KResult::Ok(t) => KResult::Ok(op(t)),
            e => KResult::err_from_u32(e.err_as_u32()),
        }
    }
}

impl<T> ops::Try for KResult<T> {
    type Output = T;
    type Residual = KResult<convert::Infallible>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        KResult::Ok(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            KResult::Ok(v) => ControlFlow::Continue(v),
            e => ControlFlow::Break(KResult::err_from_u32(e.err_as_u32())),
        }
    }
}

impl<T> ops::FromResidual<KResult<convert::Infallible>> for KResult<T> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: KResult<convert::Infallible>) -> Self {
        match residual {
            KResult::Ok(_) => KResult::InvalidArg,
            e => KResult::err_from_u32(e.err_as_u32()),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.inner.take()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = if self.inner.is_some() { 1 } else { 0 };
        (n, Some(n))
    }
}

impl<T> IntoIterator for KResult<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { inner: self.ok() }
    }
}
