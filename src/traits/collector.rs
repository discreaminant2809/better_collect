use std::ops::ControlFlow;

use crate::{Cloned, Fuse, assert_collector_by_ref};

pub trait Collector: Sized {
    /// Type of items it can collect.
    type Item;

    /// Output [`finish`](Collector::finish) yields.
    type Output;

    /// Returns a [`ControlFlow`] to command whether to stop the collection.
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()>;

    /// Finish the collection.
    fn finish(self) -> Self::Output;

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // Use `try_for_each` instead of `for` loop since the iterator may not be optimal for `for` loop
        // (e.g. `skip`, `chain`, etc.)
        items.into_iter().try_for_each(|item| self.collect(item))
    }

    #[inline]
    fn fuse(self) -> Fuse<Self> {
        Fuse::new(self)
    }

    #[inline]
    fn cloned(self) -> Cloned<Self>
    where
        Self::Item: Clone,
    {
        assert_collector_by_ref(Cloned::new(self))
    }

    // fn copied()

    // fn funnel_from_ref()

    // fn funnel()

    // fn filter()
}
