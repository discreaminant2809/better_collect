use std::ops::ControlFlow;

use crate::{Cloned, Filter, Fuse, Map, MapRef, Take, assert_collector, assert_ref_collector};

pub trait Collector: Sized {
    /// Type of items it can collect.
    type Item;

    /// Output [`finish`](Collector::finish) yields.
    type Output;

    /// Returns a [`ControlFlow`] to command whether the collector is "closed"
    /// (won't accept more items after the operation).
    ///
    /// Returns `Some(item)` if the item couldn't be collected due to not satisfying some condition,
    /// or the collector is closed.
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()>;

    /// Finish the collection.
    // Can we separate it to another trait, like `FromCollector`, so that this trait is dyn compatible?
    // NO, because the compiler will hit an evaluation recursion limit. This approach fails.
    fn finish(self) -> Self::Output;

    #[inline]
    #[allow(unused_variables)]
    fn reserve(&mut self, additional_min: usize, additional_max: Option<usize>) {}

    /// Also returns how many items were collected.
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        let (additional_min, additional_max) = items.size_hint();
        self.reserve(additional_min, additional_max);

        // Use `try_for_each` instead of `for` loop since the iterator may not be optimal for `for` loop
        // (e.g. `skip`, `chain`, etc.)
        items.try_for_each(|item| self.collect(item))
    }

    /// Can be overriden to optimize, such as [`take`](Collector::take).
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        // We don't care whether the collector breaks or not, since if it doesn't it'll have
        // completely depleted the iterator so... we just finish--nothing changed.
        let _ = self.collect_many(items);
        self.finish()
    }

    #[inline]
    fn fuse(self) -> Fuse<Self> {
        assert_collector(Fuse::new(self))
    }

    #[inline]
    fn cloned(self) -> Cloned<Self>
    where
        Self::Item: Clone,
    {
        assert_ref_collector(Cloned::new(self))
    }

    // fn copied()

    #[inline]
    fn map<E, F: FnMut(E) -> Self::Item>(self, f: F) -> Map<Self, E, F> {
        assert_collector(Map::new(self, f))
    }

    #[inline]
    fn map_ref<E, F: FnMut(&mut E) -> Self::Item>(self, f: F) -> MapRef<Self, E, F> {
        assert_ref_collector(MapRef::new(self, f))
    }

    #[inline]
    fn filter<F: FnMut(&Self::Item) -> bool>(self, pred: F) -> Filter<Self, F> {
        assert_collector(Filter::new(self, pred))
    }

    // fn modify()

    // fn filter_map()
    // fn filter_map_ref()

    // fn flat_map()

    #[inline]
    fn take(self, n: usize) -> Take<Self> {
        Take::new(self, n)
    }
    // fn take_while()

    // fn skip()
}
