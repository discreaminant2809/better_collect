use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_ref_collector};

pub struct FoldRef<A, T, F> {
    accum: A,
    f: F,
    // Since `E` appears in one of the parameters of `F`.
    _marker: PhantomData<fn(&mut T)>,
}

impl<A, T, F: FnMut(&mut A, &mut T) -> ControlFlow<()>> FoldRef<A, T, F> {
    #[inline]
    pub fn new(accum: A, f: F) -> Self {
        assert_ref_collector(FoldRef {
            accum,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, T, F: FnMut(&mut A, &mut T) -> ControlFlow<()>> Collector<T> for FoldRef<A, T, F> {
    type Output = A;

    #[inline]
    fn collect(&mut self, mut item: T) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}

impl<A, T, F: FnMut(&mut A, &mut T) -> ControlFlow<()>> RefCollector<T> for FoldRef<A, T, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item)
    }
}
