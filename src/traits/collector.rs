use std::ops::ControlFlow;

use crate::{
    Cloned, Filter, Fuse, Map, MapRef, Partition, Take, Unzip, assert_collector,
    assert_ref_collector,
};

pub trait Collector<T>: Sized {
    /// Output [`finish`](Collector::finish) yields.
    type Output;

    /// Returns a [`ControlFlow`] to command whether the collector is "closed"
    /// (won't accept more items after the operation).
    ///
    /// Returns `Some(item)` if the item couldn't be collected due to not satisfying some condition,
    /// or the collector is closed.
    fn collect(&mut self, item: T) -> ControlFlow<()>;

    /// Finish the collection.
    // Can we separate it to another trait, like `FromCollector`, so that this trait is dyn compatible?
    // NO, because the compiler will hit an evaluation recursion limit. This approach fails.
    fn finish(self) -> Self::Output;

    // #[inline]
    // fn reserve(&mut self, additional_min: usize, additional_max: Option<usize>) {
    //     let _ = (additional_min, additional_max);
    //     // Default implementation does nothing.
    // }

    // /// Hint of how many items can still be collected
    // /// before [`collect`](Collector::collect) returns [`ControlFlow::Break`]?
    // #[inline]
    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     (0, None)
    // }

    // /// Returns minimum amount of items to be collected before it's active again.
    // /// `None` means it's GUARANTEED to be permanently inactive.
    // ///
    // /// It only requires the best effort. The collector is allowed to be permanently inactive
    // /// even tho this method returns `Some(0)`. However, if the method returns `None`, the collector
    // /// is guaranteed to be permanently inactive.
    // ///
    // /// It's a hint for some adaptors (e.g. [`then`](crate::RefCollector::then)) for optimization.
    // /// However, it's up to the user to return correctly.
    // ///
    // /// The default implementation always returns `Some(0)`, meaning that the collector is
    // /// conservatively always active.
    // ///
    // /// [`Filter`] always returns `Some(0)` even though it may have inactivity periods.
    // /// It can't be confident about whether its predicate returns `true` or not for subsequent items.
    // #[inline]
    // fn inactivity_hint(&self) -> Option<usize> {
    //     Some(0)
    // }

    // /// Skips the item collection until it's active again. Skips at most `max` items.
    // ///
    // /// It should be used with [`inactive_for`](Collector::inactive_for),
    // /// and should be consistent with it.
    // #[inline]
    // fn skip_till_active(&mut self, max: Option<usize>) {
    //     let _ = max;
    //     // Default implementation does nothing.
    // }

    /// Also returns how many items were collected.
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        // let (additional_min, additional_max) = items.size_hint();
        // self.reserve(additional_min, additional_max);

        // // Block the collection beforehand. We can't affort wasting items on an inactive collector.
        // if self.inactivity_hint().is_none() {
        //     return ControlFlow::Break(());
        // }

        // Use `try_for_each` instead of `for` loop since the iterator may not be optimal for `for` loop
        // (e.g. `skip`, `chain`, etc.)
        items.into_iter().try_for_each(|item| self.collect(item))
    }

    /// Can be overriden to optimize, such as [`take`](Collector::take).
    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        // Do this instead of putting `mut` in `self` since some IDEs are stupid
        // and just put `mut self` in every generated code.
        let mut this = self;

        // We don't care whether the collector breaks or not, since if it doesn't it'll have
        // completely depleted the iterator so... we just finish--nothing changed.
        let _ = this.collect_many(items);
        this.finish()
    }

    #[inline]
    fn fuse(self) -> Fuse<Self> {
        assert_collector(Fuse::new(self))
    }

    #[inline]
    fn cloned(self) -> Cloned<Self>
    where
        T: Clone,
    {
        assert_ref_collector(Cloned::new(self))
    }

    // fn copied()

    #[inline]
    fn map<F, U>(self, f: F) -> Map<Self, U, F>
    where
        F: FnMut(U) -> T,
    {
        assert_collector(Map::new(self, f))
    }

    #[inline]
    fn map_ref<F, U>(self, f: F) -> MapRef<Self, U, F>
    where
        F: FnMut(&mut U) -> T,
    {
        assert_ref_collector(MapRef::new(self, f))
    }

    #[inline]
    fn filter<F>(self, pred: F) -> Filter<Self, F>
    where
        F: FnMut(&T) -> bool,
    {
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

    // fn step_by()

    #[inline]
    fn partition<C, F>(self, pred: F, other_if_false: C) -> Partition<Self, C, F>
    where
        C: Collector<T>,
        F: FnMut(&mut T) -> bool,
    {
        assert_collector(Partition::new(self, other_if_false, pred))
    }

    #[inline]
    fn unzip<C>(self, other: C) -> Unzip<Self, C>
    where
        C: Collector<T>,
    {
        assert_collector(Unzip::new(self, other))
    }
}
