use crate::collector::{Collector, RefCollector};

use std::ops::ControlFlow;

/// A [`RefCollector`] that copies every collected item.
///
/// This `struct` is created by [`Collector::copying()`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Copying<C>(C);

/// See [`Copying`].
#[deprecated(since = "0.3.0", note = "See `Copying`")]
pub type Copied<C> = Copying<C>;

impl<C> Copying<C> {
    pub(in crate::collector) fn new(collector: C) -> Self {
        Self(collector)
    }
}

impl<C> Collector for Copying<C>
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

impl<C> RefCollector for Copying<C>
where
    Self::Item: Copy,
    C: Collector,
{
    #[inline]
    fn collect_ref(&mut self, &mut item: &mut Self::Item) -> ControlFlow<()> {
        self.0.collect(item)
    }
}
