use crate::collector::{Collector, RefCollector};

use std::ops::ControlFlow;

/// A [`RefCollector`] that [`clone`](Clone::clone)s every collected item.
///
/// This `struct` is created by [`Collector::cloning()`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Cloning<C>(C);

/// See [`Cloning`].
#[deprecated(since = "0.3.0", note = "See `Cloning`")]
pub type Cloned<C> = Cloning<C>;

impl<C> Cloning<C> {
    pub(in crate::collector) fn new(collector: C) -> Self {
        Self(collector)
    }
}

impl<C> Collector for Cloning<C>
where
    C: Collector,
{
    type Item = C::Item;
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.0.collect(item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0.finish()
    }

    #[inline]
    fn break_hint(&self) -> bool {
        self.0.break_hint()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.0.collect_many(items)
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.0.collect_then_finish(items)
    }
}

impl<C> RefCollector for Cloning<C>
where
    Self::Item: Clone,
    C: Collector,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.0.collect(item.clone())
    }
}
