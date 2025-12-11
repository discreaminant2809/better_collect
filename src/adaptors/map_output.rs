use std::{fmt::Debug, ops::ControlFlow};

use crate::{Collector, RefCollector};

/// Creates a [`Collector`] that transforms the final accumulated result.
///
/// This `struct` is created by [`Collector::map_output()`]. See its documentation for more.
pub struct MapOutput<C, T, F>
where
    C: Collector,
    F: FnOnce(C::Output) -> T,
{
    collector: C,
    f: F,
}

impl<C, T, F> MapOutput<C, T, F>
where
    C: Collector,
    F: FnOnce(C::Output) -> T,
{
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self { collector, f }
    }
}

impl<C, T, F> Collector for MapOutput<C, T, F>
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
    fn has_stopped(&self) -> bool {
        self.collector.has_stopped()
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

impl<C, T, F> RefCollector for MapOutput<C, T, F>
where
    C: RefCollector,
    F: FnOnce(C::Output) -> T,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.collector.collect_ref(item)
    }
}

impl<C, T, F> Clone for MapOutput<C, T, F>
where
    C: Collector + Clone,
    F: FnOnce(C::Output) -> T + Clone,
{
    fn clone(&self) -> Self {
        Self {
            collector: self.collector.clone(),
            f: self.f.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.collector.clone_from(&source.collector);
        self.f.clone_from(&source.f);
    }
}

impl<C, T, F> Debug for MapOutput<C, T, F>
where
    C: Collector + Debug,
    F: FnOnce(C::Output) -> T,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapOutput")
            .field("collector", &self.collector)
            .finish()
    }
}
