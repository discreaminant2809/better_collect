use std::{fmt::Debug, ops::ControlFlow};

use crate::collector::{Collector, CollectorBase};

/// A [`Collector`] that calls a closure on each item before collecting.
///
/// This `struct` is created by [`Collector::map()`]. See its documentation for more.
#[derive(Clone)]
pub struct Map<C, F> {
    collector: C,
    f: F,
}

impl<C, F> Map<C, F> {
    pub(in crate::collector) fn new(collector: C, f: F) -> Self {
        Self { collector, f }
    }
}

impl<C, F> CollectorBase for Map<C, F>
where
    C: CollectorBase,
{
    type Output = C::Output;

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    #[inline]
    fn break_hint(&self) -> ControlFlow<()> {
        self.collector.break_hint()
    }
}

impl<C, T, U, F> Collector<T> for Map<C, F>
where
    C: Collector<U>,
    F: FnMut(T) -> U,
{
    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.collector.collect((self.f)(item))
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.collector
            .collect_many(items.into_iter().map(&mut self.f))
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().map(self.f))
    }
}

impl<C: Debug, F> Debug for Map<C, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Map")
            .field("collector", &self.collector)
            .finish()
    }
}
