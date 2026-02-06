use super::{CollectorBase, Filter, IntoCollector, Map, Partition, TakeWhile, TeeWith};
// #[cfg(feature = "unstable")]
// use super::{Nest, NestExact};

use std::ops::ControlFlow;

/// Defines what item types are accepted and how items are collected.
///
/// # Dyn Compatibility
///
/// This trait is *dyn-compatible*, meaning it can be used as a trait object.
/// You do not need to specify the [`Output`](CollectorBase::Output) type;
/// providing the item type `T` is enough.
/// The compiler will even emit a warning if you add the
/// [`Output`](CollectorBase::Output) type.
///
/// For example:
///
/// ```no_run
/// # use better_collect::prelude::*;
/// # fn foo(_:
/// &mut dyn Collector<i32>
/// # ) {}
/// ```
///
/// [`Break(())`]: std::ops::ControlFlow::Break
pub trait Collector<T>: CollectorBase {
    /// Collects an item and returns a [`ControlFlow`] indicating whether
    /// the collector has stopped accumulating right after this operation.
    ///
    /// Return [`Continue(())`] to indicate the collector can still accumulate more items,
    /// or [`Break(())`] if it will not anymore and hence should no longer be fed further.
    ///
    /// This is analogous to [`Iterator::next()`], which returns an item (instead of collecting one)
    /// and signals with [`None`] whenever it finishes.
    ///
    /// Implementors should inform the caller about it as early as possible.
    /// This can usually be upheld, but not always.
    /// Some collectors, such as [`take(0)`](CollectorBase::take) and [`take_while()`],
    /// only know when they are done after collecting an item, which might be too late
    /// if the item cannot be “afforded” and is lost forever.
    /// In this case, call [`break_hint()`](CollectorBase::break_hint)
    /// **once and only once** before collecting (see its documentation to use it correctly).
    /// For "infinite" collectors (like most collections), this is not an issue
    /// since they can simply return  [`Continue(())`] every time.
    ///
    /// If the collector is uncertain, like "maybe I won’t accumulate… uh, fine, I will,"
    /// it is recommended to just return [`Continue(())`].
    /// For example, [`filter()`](Collector::filter) might skip some items it collects,
    /// but still returns [`Continue(())`] as long as the underlying collector can still accumulate.
    /// The filter just denies "undesirable" items, not signal termination
    /// (this is the job of [`take_while()`] instead).
    ///
    /// Collectors with limited capacity (e.g., a `Vec` stored on the stack) will eventually
    /// return [`Break(())`] once full, right after the last item is accumulated.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![].into_collector().take(3); // only takes 3 items
    ///
    /// // It has not reached its 3-item quota yet.
    /// assert!(collector.collect(1).is_continue());
    /// assert!(collector.collect(2).is_continue());
    ///
    /// // After collecting `3`, it meets the quota, so it signals `Break` immediately.
    /// assert!(collector.collect(3).is_break());
    /// # // Internal assertion.
    /// # assert!(collector.collect(4).is_break());
    ///
    /// assert_eq!(collector.finish(), [1, 2, 3]);
    /// ```
    ///
    /// Most collectors can accumulate indefinitely.
    ///
    /// ```
    /// use better_collect::{prelude::*, iter::Last};
    ///
    /// let mut last = Last::new();
    /// for num in 0..100 {
    ///     assert!(last.collect(num).is_continue(), "cannot collect {num}");
    /// }
    ///
    /// assert_eq!(last.finish(), Some(99));
    /// ```
    ///
    /// [`Continue(())`]: ControlFlow::Continue
    /// [`Break(())`]: ControlFlow::Break
    /// [`take_while()`]: Collector::take_while
    fn collect(&mut self, item: T) -> ControlFlow<()>;

    /// Collects items from an iterator and returns a [`ControlFlow`] indicating whether
    /// the collector has stopped collecting right after this operation.
    ///
    /// This method can be overridden for optimization and/or to avoid consuming one item prematurely.
    /// Implementors may choose a more efficient way to consume an iterator than a simple `for` loop
    /// ([`Iterator`] offers many alternative consumption methods), depending on the collector’s needs.
    ///
    /// Unlike [`collect()`](Self::collect), callers are **note** required to check for
    /// [`break_hint()`](CollectorBase::break_hint)
    /// and the implementors should guard against empty iterators.
    /// As a result, `collector.collect_many(empty_iter)` is an alternative
    /// way to check whether this collector has stopped accumulating.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![1, 2].into_collector();
    /// collector.collect_many([3, 4, 5]);
    ///
    /// assert_eq!(collector.finish(), [1, 2, 3, 4, 5]);
    /// ```
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()>
    where
        Self: Sized,
    {
        self.break_hint()?;

        // Use `try_for_each` instead of `for` loop since the iterator may not be optimal for `for` loop
        // (e.g. `skip`, `chain`, etc.)
        items.into_iter().try_for_each(|item| self.collect(item))
    }

    /// Collects items from an iterator, consumes the collector, and produces the accumulated result.
    ///
    /// This is equivalent to calling [`collect_many`](Collector::collect_many)  
    /// followed by [`finish`](Collector::finish) (which is the default implementation),
    /// but it can be overridden for optimization (e.g., to skip tracking internal state)
    /// because the collector will be dropped anyway.
    /// For instance, [`take()`](Collector::take) overrides this method to avoid tracking
    /// how many items have been collected.
    ///
    /// Unlike [`collect()`](Self::collect), callers are **not** required to check for
    /// [`break_hint()`](CollectorBase::break_hint)
    /// and the implementors should guard against empty iterators.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use better_collect::prelude::*;
    ///
    /// let collector = vec![1, 2].into_collector();
    ///
    /// assert_eq!(collector.collect_then_finish([3, 4, 5]), [1, 2, 3, 4, 5]);
    /// ```
    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output
    where
        Self: Sized,
    {
        // Do this instead of putting `mut` in `self` since some IDEs are stupid
        // and just put `mut self` in every generated code.
        let mut this = self;

        // We don't care whether the collector breaks or not, since if it doesn't it'll have
        // completely depleted the iterator so... we just finish--nothing changed.
        let _ = this.collect_many(items);
        this.finish()
    }

    /// Creates a collector that calls a closure on each item before collecting.
    ///
    /// This is used when you need a collector that collects `U`,
    /// but you have a collector that collects `T`. In that case,
    /// you can use `map()` to transform `U` into `T` before passing it along.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![].into_collector().map(|num| num * num);
    ///
    /// assert!(collector.collect_many(1..=5).is_continue());
    ///
    /// assert_eq!(collector.finish(), [1, 4, 9, 16, 25]);
    /// ```
    ///
    /// If you have multiple collectors with different item types, this adaptor bridges them.
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let (_strings, lens) = ["a", "bcd", "ef"]
    ///     .into_iter()
    ///     .feed_into(
    ///         "".to_owned()
    ///             .into_concat()
    ///             // Limitation: type annotation may be needed.
    ///             .tee(vec![].into_collector().map(|s: &str| s.len()))
    ///     );
    ///
    /// assert_eq!(lens, [1, 3, 2]);
    /// ```
    #[inline]
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(U) -> T,
    {
        assert_collector::<_, U>(Map::new(self, f))
    }

    /// Creates a collector that uses a closure to determine whether an item should be accumulated.
    ///
    /// The underlying collector only collects items for which the given predicate returns `true`.
    ///
    /// Note that even if an item is not collected, this adaptor will still return
    /// [`Continue`] as long as the underlying collector does. If you want the collector to stop
    /// after the first `false`, consider using [`take_while()`](Collector::take_while) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![]
    ///     .into_collector()
    ///     .filter(|&x| x % 2 == 0);
    ///
    /// assert!(collector.collect(2).is_continue());
    /// assert!(collector.collect(4).is_continue());
    /// assert!(collector.collect(0).is_continue());
    ///
    /// // Still `Continue` even if an item doesn’t satisfy the predicate.
    /// assert!(collector.collect(1).is_continue());
    ///
    /// assert_eq!(collector.finish(), [2, 4, 0]);
    /// ```
    #[inline]
    fn filter<F>(self, pred: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: FnMut(&T) -> bool,
    {
        assert_collector::<_, T>(Filter::new(self, pred))
    }

    /// Creates a collector that accumulates items as long as a predicate returns `true`.
    ///
    /// `take_while()` collects items until it encounters one for which the predicate returns `false`.
    /// Conceptually, that item and all subsequent ones will **not** be accumulated.
    /// However, you should ensure that you do not feed more items after it has signaled
    /// a stop.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = "".to_owned()
    ///     .into_concat()
    ///     .take_while(|&s| s != "stop");
    ///
    /// assert!(collector.collect("abc").is_continue());
    /// assert!(collector.collect("def").is_continue());
    ///
    /// // Immediately stops after "stop".
    /// assert!(collector.collect("stop").is_break());
    ///
    /// assert_eq!(collector.finish(), "abcdef");
    /// ```
    fn take_while<F>(self, pred: F) -> TakeWhile<Self, F>
    where
        Self: Sized,
        F: FnMut(&T) -> bool,
    {
        assert_collector::<_, T>(TakeWhile::new(self, pred))
    }

    // fn step_by()

    /// Creates a collector that distributes items between two collectors based on a predicate.
    ///
    /// Items for which the predicate returns `true` are sent to the first collector,
    /// and those for which it returns `false` go to the second collector.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let collector = vec![]
    ///     .into_collector()
    ///     .partition(|&mut x| x % 2 == 0, vec![]);
    /// let (evens, odds) = collector.collect_then_finish(-5..5);
    ///
    /// assert_eq!(evens, [-4, -2, 0, 2, 4]);
    /// assert_eq!(odds, [-5, -3, -1, 1, 3]);
    /// ```
    #[inline]
    fn partition<C, F>(self, pred: F, other_if_false: C) -> Partition<Self, C::IntoCollector, F>
    where
        Self: Sized,
        C: IntoCollector<T>,
        F: FnMut(&mut T) -> bool,
    {
        assert_collector::<_, T>(Partition::new(self, other_if_false.into_collector(), pred))
    }

    /// Creates a collector that lets both collectors collect the same item.
    ///
    /// For each item collected, the first collector collects the item
    /// mapped by a given closure before the second collector collects it.
    /// If the second collector stops accumulating, the item will **not**
    /// be mapped, and instead is fed directly into the first collector.
    ///
    /// `tee_with()` only stops when **both** collectors have stopped.
    ///
    /// If the item type of this adapter is `T`, the first collector must implement
    /// [`Collector<T>`](super::Collector) and [`Collector<U>`](super::Collector),
    /// and the second collector must implement [`Collector<T>`](super::Collector).
    /// Since many collectors do not collect two or more types of items,
    /// `U` is effectively also `T` in this case.
    ///
    /// The [`Output`](CollectorBase::Output) is a tuple containing the outputs of
    /// both underlying collectors, in order.
    ///
    /// See the [module-level documentation](crate::collector) for
    /// when this adapter is used and other variants of `tee` adapters.
    #[inline]
    #[cfg(feature = "unstable")]
    fn tee_with<C, F, U>(self, f: F, other: C) -> TeeWith<Self, C::IntoCollector, F>
    where
        Self: Collector<U> + Sized,
        C: IntoCollector<T>,
        F: FnMut(&mut T) -> U,
    {
        assert_collector::<_, T>(TeeWith::new(self, other.into_collector(), f))
    }

    // /// Creates a [`Collector`] that collects all outputs produced by an inner collector.
    // ///
    // /// The inner collector collects items first until it stops accumulating,
    // /// then, the outer collector collects the output produced by the inner collector,
    // /// then repeat.
    // ///
    // /// The inner collector must implement [`Clone`]. Also, it should be finite
    // /// so that the outer can collect more, or else the outer will be stuck with
    // /// one output forever.
    // ///
    // /// This version collects the unfinished inner (the remainder), if any,
    // /// after calling [`finish()`] or [`collect_then_finish()`].
    // /// Hence, this adaptor is not "exact," similar to [`[_]::chunks()`](slice::chunks).
    // /// Use [`nest_exact()`](Collector::nest_exact) if you do not care about the remainder,
    // /// since the exact verion is generally faster.
    // ///
    // /// This also implements [`RefCollector`] if the inner collector does.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use better_collect::prelude::*;
    // ///
    // /// let mut collector = vec![]
    // ///     .into_collector()
    // ///     .nest(vec![].into_collector().take(3));
    // ///
    // /// assert!(collector.collect_many(1..=11).is_continue());
    // ///
    // /// assert_eq!(
    // ///     collector.finish(),
    // ///     [
    // ///         vec![1, 2, 3],
    // ///         vec![4, 5, 6],
    // ///         vec![7, 8, 9],
    // ///         vec![10, 11],
    // ///     ],
    // /// );
    // /// ```
    // ///
    // /// [`RefCollector`]: crate::collector::RefCollector
    // /// [`finish()`]: Collector::finish
    // /// [`collect_then_finish()`]: Collector::collect_then_finish
    // #[cfg(feature = "unstable")]
    // fn nest<C>(self, inner: C) -> Nest<Self, C::IntoCollector>
    // where
    //     Self: Sized,
    //     C: IntoCollector<IntoCollector: Collector<Self::Output> + Clone>,
    // {
    //     assert_collector::<_, T>(Nest::new(self, inner.into_collector()))
    // }

    // /// Creates a [`Collector`] that collects all outputs produced by an inner collector.
    // ///
    // /// The inner collector collects items first until it stops accumulating,
    // /// then, the outer collector collects the output produced by the inner collector,
    // /// then repeat.
    // ///
    // /// The inner collector must implement [`Clone`]. Also, it should be finite
    // /// so that the outer can collect more, or else the outer will be stuck with
    // /// one output forever.
    // ///
    // /// This version will only collect all the inners that has stopped accumulating.
    // /// Any unfinished inner (the remainder) is discarded after calling
    // /// [`finish()`] or [`collect_then_finish()`].
    // /// Hence, this adaptor is "exact," similar to [`[_]::chunks_exact()`](slice::chunks_exact).
    // /// Since the implementation is simpler, this adaptor is generally faster.
    // /// Use [`nest()`](Collector::nest) if you care about the remainder.
    // ///
    // /// This also implements [`RefCollector`] if the inner collector does.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use better_collect::prelude::*;
    // ///
    // /// let mut collector = vec![]
    // ///     .into_collector()
    // ///     .nest_exact(vec![].into_collector().take(3));
    // ///
    // /// assert!(collector.collect_many(1..=11).is_continue());
    // ///
    // /// assert_eq!(
    // ///     collector.finish(),
    // ///     [
    // ///         [1, 2, 3],
    // ///         [4, 5, 6],
    // ///         [7, 8, 9],
    // ///     ],
    // /// );
    // /// ```
    // ///
    // /// [`RefCollector`]: crate::collector::RefCollector
    // /// [`finish()`]: Collector::finish
    // /// [`collect_then_finish()`]: Collector::collect_then_finish
    // #[cfg(feature = "unstable")]
    // fn nest_exact<C>(self, inner: C) -> NestExact<Self, C::IntoCollector>
    // where
    //     Self: Sized,
    //     C: IntoCollector<IntoCollector: Collector<Self::Output> + Clone>,
    // {
    //     assert_collector::<_, T>(NestExact::new(self, inner.into_collector()))
    // }
}

/// A mutable reference to a collect produce nothing.
///
/// This is useful when you *just* want to feed items to a collector without
/// finishing it.
impl<C, T> Collector<T> for &mut C
where
    C: Collector<T>,
{
    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        C::collect(self, item)
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        // FIXED: specialization for unsized type.
        // We can't add `?Sized` to the bound of `C` because this method requires `Sized`.
        C::collect_many(self, items)
    }

    // The default implementation for `collect_then_finish()` is sufficient.
}

macro_rules! dyn_impl {
    ($($traits:ident)*) => {
        impl<'a, T> Collector<T> for &mut (dyn Collector<T> $(+ $traits)* + 'a) {
            #[inline]
            fn collect(&mut self, item: T) -> ControlFlow<()> {
                <dyn Collector<T>>::collect(*self, item)
            }

            // The default implementations are sufficient.
        }
    };
}

dyn_impl!();
dyn_impl!(Send);
dyn_impl!(Sync);
dyn_impl!(Send Sync);

// `Output` shouldn't be required to be specified.
fn _dyn_compatible<T>(_: &mut dyn Collector<T>) {}

#[inline(always)]
fn assert_collector<C, T>(collector: C) -> C
where
    C: Collector<T>,
{
    collector
}
