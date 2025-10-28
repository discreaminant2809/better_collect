use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

pub struct Funnel<C, T, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(&mut T)>,
}

impl<C, T, F> Funnel<C, T, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<T, U, C: RefCollector<U>, F: FnMut(&mut T) -> &mut U> Collector<T> for Funnel<C, T, F> {
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, mut item: T) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    // Not doable...
    // fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
    //     self.collector
    //         .collect_many(items.into_iter().map(|mut item| (self.f)(&mut item))) // ...due to this.
    // }
}

impl<T, U, C: RefCollector<U>, F: FnMut(&mut T) -> &mut U> RefCollector<T> for Funnel<C, T, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        self.collector.collect_ref((self.f)(item))
    }
}
