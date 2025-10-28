use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, assert_collector};

pub struct Fold<A, T, F> {
    accum: A,
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<A, T, F: FnMut(&mut A, T) -> ControlFlow<()>> Fold<A, T, F> {
    #[inline]
    pub fn new(accum: A, f: F) -> Self {
        assert_collector(Fold {
            accum,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, T, F: FnMut(&mut A, T) -> ControlFlow<()>> Collector<T> for Fold<A, T, F> {
    type Output = A;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}
