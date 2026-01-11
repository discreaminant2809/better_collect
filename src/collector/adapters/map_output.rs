use std::{fmt::Debug, ops::ControlFlow};

use crate::collector::{Collector, RefCollector};

/// Creates a [`Collector`] that transforms the final accumulated result.
///
/// This `struct` is created by [`Collector::map_output()`]. See its documentation for more.
#[derive(Clone)]
pub struct MapOutput<C, F> {
    collector: C,
    f: F,
}

impl<C, F> MapOutput<C, F> {
    pub(in crate::collector) fn new(collector: C, f: F) -> Self {
        Self { collector, f }
    }
}

impl<C, T, F> Collector for MapOutput<C, F>
where
    C: Collector,
    F: FnOnce(C::Output) -> T,
{
    type Item = C::Item;

    type Output = T;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.collector.collect(item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        (self.f)(self.collector.finish())
    }

    #[inline]
    fn break_hint(&self) -> bool {
        self.collector.break_hint()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.collector.collect_many(items)
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        (self.f)(self.collector.collect_then_finish(items))
    }
}

impl<C, T, F> RefCollector for MapOutput<C, F>
where
    C: RefCollector,
    F: FnOnce(C::Output) -> T,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.collector.collect_ref(item)
    }
}

impl<C, F> Debug for MapOutput<C, F>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapOutput")
            .field("collector", &self.collector)
            .finish()
    }
}
