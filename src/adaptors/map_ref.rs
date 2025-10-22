use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

pub struct MapRef<C, E, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(&mut E)>,
}

impl<C, E, F> MapRef<C, E, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<E, C: Collector, F: FnMut(&mut E) -> C::Item> Collector for MapRef<C, E, F> {
    type Item = E;

    type Output = C::Output;

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.collector.size_hint()
    }

    #[inline]
    fn reserve(&mut self, additional_min: usize, additional_max: Option<usize>) {
        self.collector.reserve(additional_min, additional_max);
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.collector
            .collect_many(items.into_iter().map(|mut item| (self.f)(&mut item)))
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().map(move |mut item| (self.f)(&mut item)))
    }
}

impl<E, C: Collector, F: FnMut(&mut E) -> C::Item> RefCollector for MapRef<C, E, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.collector.collect((self.f)(item))
    }
}
