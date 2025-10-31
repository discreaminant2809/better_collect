use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, assert_collector};

pub struct Fold<A, T, F> {
    accum: A,
    f: F,
    // Needed, or else the compiler will complain about "unconstraint generics."
    // Since we use `T` in the function params, it's logical to use `PhantomData` like this.
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

impl<A, T, F: FnMut(&mut A, T) -> ControlFlow<()>> Collector for Fold<A, T, F> {
    type Item = T;
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
