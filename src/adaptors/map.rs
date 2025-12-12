use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::Collector;

/// A [`Collector`] that calls a closure on each item before collecting.
///
/// This `struct` is created by [`Collector::map()`]. See its documentation for more.
pub struct Map<C, T, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<C, T, F> Map<C, T, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<T, C, F> Collector for Map<C, T, F>
where
    C: Collector,
    F: FnMut(T) -> C::Item,
{
    type Item = T;
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.collector.collect((self.f)(item))
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    #[inline]
    fn break_hint(&self) -> bool {
        self.collector.break_hint()
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.collector
            .collect_many(items.into_iter().map(&mut self.f))
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().map(self.f))
    }
}

impl<C: Clone, T, F: Clone> Clone for Map<C, T, F> {
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

impl<C: Debug, T, F> Debug for Map<C, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Map")
            .field("collector", &self.collector)
            .finish()
    }
}
