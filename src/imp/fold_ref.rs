use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_ref_collector};

pub struct FoldRef<A, E, F: FnMut(&mut A, &mut E) -> ControlFlow<()>> {
    accum: A,
    f: F,
    // Since `E` appears in one of the parameters of `F`.
    _marker: PhantomData<fn(E)>,
}

impl<A, E, F: FnMut(&mut A, &mut E) -> ControlFlow<()>> FoldRef<A, E, F> {
    #[inline]
    pub fn new(accum: A, f: F) -> Self {
        assert_ref_collector(FoldRef {
            accum,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, E, F: FnMut(&mut A, &mut E) -> ControlFlow<()>> Collector for FoldRef<A, E, F> {
    type Item = E;

    type Output = A;

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}

impl<A, E, F: FnMut(&mut A, &mut E) -> ControlFlow<()>> RefCollector for FoldRef<A, E, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item)
    }
}
