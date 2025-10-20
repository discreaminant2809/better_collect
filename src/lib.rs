#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(not(feature = "std"))]
extern crate core as std;

mod adaptors;
mod imp;

pub use adaptors::*;
pub use imp::*;

use std::ops::ControlFlow;

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

pub trait RefCollector: Sized {
    /// Type of items it can collect.
    type Item;

    /// Output [`finish`](Collector::finish) yields.
    type Output;

    /// Returns a [`ControlFlow`] to command whether to stop the collection.
    fn collect(&mut self, item: &mut Self::Item) -> ControlFlow<()>;

    /// Finish the collection.
    fn finish(self) -> Self::Output;

    /// The reason we require owned items instead of references is because ultimately, collectors shouldn't be
    /// used directly, and it's used alongside with [`Collector`] which expects this parameter.
    /// It's not possible, yet, to convert an iterator yielding owned items to an iterator yielding references.
    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // Use `try_for_each` instead of `for` loop since the iterator may not be optimal for `for` loop
        // (e.g. `skip`, `chain`, etc.)
        items
            .into_iter()
            .try_for_each(|mut item| self.collect(&mut item))
    }

    #[inline]
    fn then<C: RefCollector<Item = Self::Item>>(self, other: C) -> Then<Self, C> {
        assert_collector_by_ref(Then::new(self, other))
    }

    #[inline]
    fn then_owned<C: Collector<Item = Self::Item>>(self, other: C) -> Then<Self, C> {
        assert_collector(Then::new(self, other))
    }

    #[inline]
    fn owned(self) -> Owned<Self> {
        assert_collector(Owned::new(self))
    }

    // No T -> &U variant. See E0582
    // fn ref_map()

    // fn ref_filter()

    // fn ref_inspect()
}

pub trait BetterCollect: Iterator {
    fn better_collect<C: Collector<Item = Self::Item>>(&mut self, mut collector: C) -> C::Output {
        // We don't care whether the collector breaks or not, since if it doesn't it'll have
        // completely depleted the iterator so... we just finish--nothing changed.
        let _ = collector.collect_many(self);
        collector.finish()
    }
}

impl<I: Iterator> BetterCollect for I {}

#[inline(always)]
fn assert_collector<C: Collector>(collector: C) -> C {
    collector
}

#[inline(always)]
fn assert_collector_by_ref<C: RefCollector>(collector: C) -> C {
    collector
}

#[cfg(test)]
mod tests {
    use crate::{BetterCollect, Collector, RefCollector};

    #[test]
    fn then() {
        let arr = [1, 2, 3];
        let (arr1, arr2) = arr.into_iter().better_collect(vec![].then(vec![]));
        assert_eq!(arr1, arr);
        assert_eq!(arr2, arr);

        let arr = ["1", "2", "3"];
        let (arr1, arr2) = ["1", "2", "3"]
            .into_iter()
            .map(String::from)
            .better_collect(vec![].cloned().then_owned(vec![]));
        assert_eq!(arr1, arr);
        assert_eq!(arr2, arr);
    }
}
