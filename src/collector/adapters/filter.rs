use crate::collector::{Collector, RefCollector};

use std::{fmt::Debug, ops::ControlFlow};

/// A [`Collector`] that uses a closure to determine whether an item should be collected.
///
/// This `struct` is created by [`Collector::filter()`]. See its documentation for more.
#[derive(Clone)]
pub struct Filter<C, F> {
    collector: C,
    pred: F,
}

impl<C, F> Filter<C, F> {
    pub(in crate::collector) fn new(collector: C, pred: F) -> Self {
        Self { collector, pred }
    }
}

impl<C, F> Collector for Filter<C, F>
where
    C: Collector,
    F: FnMut(&C::Item) -> bool,
{
    type Item = C::Item;
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        if (self.pred)(&item) {
            self.collector.collect(item)
        } else {
            ControlFlow::Continue(())
        }
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
            .collect_many(items.into_iter().filter(&mut self.pred))
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().filter(self.pred))
    }
}

impl<C, F> RefCollector for Filter<C, F>
where
    C: RefCollector,
    F: FnMut(&C::Item) -> bool,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        if (self.pred)(item) {
            self.collector.collect_ref(item)
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl<C: Debug, F> Debug for Filter<C, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Filter")
            .field("collector", &self.collector)
            .finish()
    }
}
