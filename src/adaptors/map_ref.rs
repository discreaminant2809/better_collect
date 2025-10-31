use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

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

impl<T, C, F> Collector for MapRef<C, T, F>
where
    C: Collector,
    F: FnMut(&mut T) -> C::Item,
{
    type Item = T;
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, mut item: T) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
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

impl<T, C, F> RefCollector for MapRef<C, T, F>
where
    C: Collector,
    F: FnMut(&mut T) -> C::Item,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.collector.collect((self.f)(item))
    }
}

impl<C: Clone, T, F: Clone> Clone for MapRef<C, T, F> {
    fn clone(&self) -> Self {
        Self {
            collector: self.collector.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.collector.clone_from(&source.collector);
        self.f.clone_from(&source.f);
    }
}

impl<C: Debug, T, F> Debug for MapRef<C, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapRef")
            .field("collector", &self.collector)
            .finish()
    }
}
