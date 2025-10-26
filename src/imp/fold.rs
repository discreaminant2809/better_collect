use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, assert_collector};

pub struct Fold<A, E, F> {
    accum: A,
    f: F,
    _marker: PhantomData<fn(E)>,
}

impl<A, E, F: FnMut(&mut A, E) -> ControlFlow<()>> Fold<A, E, F> {
    #[inline]
    pub fn new(accum: A, f: F) -> Self {
        assert_collector(Fold {
            accum,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, E, F: FnMut(&mut A, E) -> ControlFlow<()>> Collector for Fold<A, E, F> {
    type Item = E;

    type Output = A;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}
