use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

#[inline]
pub fn fold<A, E, F: FnMut(&mut A, E) -> ControlFlow<()>>(accum: A, f: F) -> Fold<A, E, F> {
    Fold::new(accum, f)
}

pub struct Fold<A, E, F: FnMut(&mut A, E) -> ControlFlow<()>> {
    accum: A,
    f: F,
    // Since `E` appears in one of the parameters of `F`.
    _marker: PhantomData<fn(E)>,
}

impl<A, E, F: FnMut(&mut A, E) -> ControlFlow<()>> Fold<A, E, F> {
    fn new(accum: A, f: F) -> Self {
        Self {
            accum,
            f,
            _marker: PhantomData,
        }
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

impl<A, E, F: FnMut(&mut A, &mut E) -> ControlFlow<()>> RefCollector for Fold<A, &mut E, F> {
    type Item = E;

    type Output = A;

    #[inline]
    fn collect(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}
