use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

pub struct MapRef<C, T, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(&mut T)>,
}

impl<C, T, F> MapRef<C, T, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<T, U, C: Collector<U>, F: FnMut(&mut T) -> U> Collector<T> for MapRef<C, T, F> {
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, mut item: T) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.collector
            .collect_many(items.into_iter().map(|mut item| (self.f)(&mut item)))
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().map(move |mut item| (self.f)(&mut item)))
    }
}

impl<T, U, C: Collector<U>, F: FnMut(&mut T) -> U> RefCollector<T> for MapRef<C, T, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        self.collector.collect((self.f)(item))
    }
}
