use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, assert_collector};

#[inline]
pub fn fold<A, E, F: FnMut(&mut A, E) -> ControlFlow<()>>(accum: A, f: F) -> Fold<A, E, F> {
    assert_collector(Fold {
        accum,
        f,
        _marker: PhantomData,
    })
}

pub struct Fold<A, E, F: FnMut(&mut A, E) -> ControlFlow<()>> {
    accum: A,
    f: F,
    // Since `E` appears in one of the parameters of `F`.
    _marker: PhantomData<fn(E)>,
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
