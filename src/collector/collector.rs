use super::{CollectorBase, Filter, IntoCollector, Map, Partition, TakeWhile, Unbatching};
// #[cfg(feature = "unstable")]
// use super::{Nest, NestExact};

use std::ops::ControlFlow;

/// Collects items and produces a final output.
///
/// This trait requires two core methods:
///
/// - [`collect`](Collector::collect): consumes an item and returns whether the collector continues
///   accumulating further items *after* this operation.
/// - [`finish`](Collector::finish): consumes the collector and returns the accumulated result.
///
/// # Implementing
///
/// For a simple collector, define the output type you want to produce, wrap it in a struct,
/// and implement this trait for that struct.
/// You may also override methods like [`collect_many`](Collector::collect_many) for optimizations.
///
/// # Panics
///
/// Unless stated otherwise by the collector’s implementation, after any of
/// [`Collector::collect()`], [`Collector::collect_many()`], and
/// [`RefCollector::collect_ref()`](crate::collector::RefCollector::collect_ref)
/// have returned [`Break(())`] once,
/// or [`Collector::break_hint()`] has returned `true` once,
/// behaviors of subsequent calls to any method other than
/// [`finish()`](Collector::finish) are unspecified.
/// They may panic, overflow, or even resume accumulation
/// (similar to how [`Iterator::next()`] might yield again after returning [`None`]).
/// Callers should generally call [`finish()`](Collector::finish) once a collector
/// has signaled a stop.
/// If this invariant cannot be upheld, wrap it with [`fuse()`](Collector::fuse).
///
/// This looseness allows for optimizations (for example, omitting an internal "stopped” flag).
///
/// Although the behavior is unspecified, none of the aforementioned methods are `unsafe`.
/// Implementors must **not** cause memory corruption, undefined behavior,
/// or any other safety violations, and callers must **not** rely on such outcomes.
///
/// # Dyn Compatibility
///
/// This trait is *dyn-compatible*, meaning it can be used as a trait object.
/// You do not need to specify the [`Output`](crate::collector::Collector::Output) type;
/// providing the [`Item`](crate::collector::Collector::Item) type is enough.
/// The compiler will even emit a warning if you add the
/// [`Output`](crate::collector::Collector::Output) type.
///
/// For example:
///
/// ```no_run
/// # use better_collect::prelude::*;
/// # fn foo(_:
/// &mut dyn Collector<Item = i32>
/// # ) {}
/// ```
///
/// # Limitations
///
/// In some cases, you may need to explicitly annotate the parameter types in closures,
/// especially for adaptors that take generic functions.
/// This is due to current limitations in Rust’s type inference for closure parameters.
///
/// # Example
///
/// Suppose we are building a tokenizer to process text for an NLP model.
/// We will skip all complicated details for now and simply collect every word we see.
///
/// ```
/// use std::{ops::ControlFlow, collections::HashMap};
/// use better_collect::prelude::*;
///
/// #[derive(Default)]
/// struct Tokenizer {
///     indices: HashMap<String, usize>,
///     words: Vec<String>,
/// }
///
/// impl Tokenizer {
///     fn tokenize(&self, sentence: &str) -> Vec<usize> {
///         sentence
///             .split_whitespace()
///             .map(|word| self.indices.get(word).copied().unwrap_or(0))
///             .collect()
///     }
/// }
///
/// impl Collector for Tokenizer {
///     type Item = String;
///     // For now, for simplicity, we just return the struct itself.
///     type Output = Self;
///
///     fn collect(&mut self, word: Self::Item) -> ControlFlow<()> {
///         self.indices
///             .entry(word)
///             .or_insert_with_key(|word| {
///                 self.words.push(word.clone());
///                 // Reserve index 0 for out-of-vocabulary words.
///                 self.words.len()
///             });
///
///         // Tokenizer never stops accumulating.
///         ControlFlow::Continue(())
///     }
///
///     fn finish(self) -> Self::Output {
///         // Just return itself.
///         self
///     }
/// }
///
/// let sentence = "the noble and the singer";
/// let tokenizer = sentence
///     .split_whitespace()
///     .map(String::from)
///     .feed_into(Tokenizer::default());
///
/// // "the" should only appear once.
/// assert_eq!(tokenizer.words, ["the", "noble", "and", "singer"]);
/// assert_eq!(tokenizer.tokenize("the singer and the swordswoman"), [1, 4, 3, 1, 0]);
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
    /// Some collectors, such as [`take(0)`](Collector::take) and [`take_while()`],
    /// only know when they are done after collecting an item, which might be too late
    /// if the item cannot be “afforded” and is lost forever.
    /// In this case, call [`break_hint()`](Collector::break_hint) before collecting
    /// (see its documentation to use it correctly).
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

    /// Creates a [`Collector`] that calls a closure on each item before collecting.
    ///
    /// This is used when a [`combine`] chain expects to collect `T`,
    /// but you have a collector that collects `U`. In that case,
    /// you can use `map()` to transform `U` into `T` before passing it along.
    ///
    /// Since it does not implement [`RefCollector`], this adaptor should be used
    /// on the final collector in a [`combine`] chain, or adapted into a [`RefCollector`]
    /// using the appropriate adaptor.
    /// If you find yourself writing `map().cloning()` or `map().copying()`,
    /// consider using [`map_ref()`](Collector::map_ref) instead, which avoids unnecessary cloning.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// // Collect the first 5 squared numbers.
    /// let collector_squares = (1..=5)
    ///     .feed_into(vec![].into_collector().map(|num| num * num));
    ///
    /// assert_eq!(collector_squares, [1, 4, 9, 16, 25]);
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
    ///             .combine(vec![].into_collector().map(|s: &str| s.len()))
    ///     );
    ///
    /// assert_eq!(lens, [1, 3, 2]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`combine`]: crate::collector::RefCollector::combine
    #[inline]
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(U) -> T,
    {
        assert_collector::<_, U>(Map::new(self, f))
    }

    /// Creates a [`Collector`] that uses a closure to determine whether an item should be accumulated.
    ///
    /// The underlying collector only collects items for which the given predicate returns `true`.
    ///
    /// This also implements [`RefCollector`] if the underlying collector does.
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
    /// let mut collector = vec![].into_collector().filter(|&x| x % 2 == 0);
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
    ///
    /// The adaptor also implements [`RefCollector`] if the underlying collector does.
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let (evens, negs) = (-5..=5)
    ///     .feed_into(
    ///         vec![]
    ///             .into_collector()
    ///             .filter(|&x| x % 2 == 0)
    ///             // Since `Vec<i32>::IntoCollector` implements `RefCollector`,
    ///             // this adaptor does too!
    ///             .combine(vec![].into_collector().filter(|&x| x < 0))
    ///     );
    ///
    /// assert_eq!(evens, [-4, -2, 0, 2, 4]);
    /// assert_eq!(negs, [-5, -4, -3, -2, -1]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`Continue`]: std::ops::ControlFlow::Continue
    /// [`Break`]: std::ops::ControlFlow::Break
    #[inline]
    fn filter<F>(self, pred: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: FnMut(&T) -> bool,
    {
        assert_collector::<_, T>(Filter::new(self, pred))
    }

    /// Creates a [`Collector`] that accumulates items as long as a predicate returns `true`.
    ///
    /// `take_while()` collects items until it encounters one for which the predicate returns `false`.
    /// Conceptually, that item and all subsequent ones will **not** be accumulated.
    /// However, you should ensure that you do not feed more items after it has signaled
    /// a stop.
    ///
    /// This also implements [`RefCollector`] if the underlying collector does.
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
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    fn take_while<F>(self, pred: F) -> TakeWhile<Self, F>
    where
        Self: Sized,
        F: FnMut(&T) -> bool,
    {
        assert_collector::<_, T>(TakeWhile::new(self, pred))
    }

    // fn step_by()

    /// Creates a [`Collector`] that distributes items between two collectors based on a predicate.
    ///
    /// Items for which the predicate returns `true` are sent to the first collector,
    /// and those for which it returns `false` go to the second collector.
    ///
    /// This adaptor also implements [`RefCollector`] if both underlying collectors do.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let collector = vec![].into_collector().partition(|&mut x| x % 2 == 0, vec![]);
    /// let (evens, odds) = collector.collect_then_finish(-5..5);
    ///
    /// assert_eq!(evens, [-4, -2, 0, 2, 4]);
    /// assert_eq!(odds, [-5, -3, -1, 1, 3]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    #[inline]
    fn partition<C, F>(self, pred: F, other_if_false: C) -> Partition<Self, C::IntoCollector, F>
    where
        Self: Sized,
        C: IntoCollector<IntoCollector: Collector<T>>,
        F: FnMut(&mut T) -> bool,
    {
        assert_collector::<_, T>(Partition::new(self, other_if_false.into_collector(), pred))
    }

    /// Creates a [`Collector`] with a custom collection logic.
    ///
    /// This adaptor is useful for behaviors that cannot be expressed
    /// through existing adaptors without cloning or intermediate allocations.
    ///
    /// Since it does not implement [`RefCollector`], this adaptor should be used
    /// on the final collector in a [`combine`] chain, or adapted into a [`RefCollector`]
    /// using the appropriate adaptor.
    /// If you find yourself writing `unbatching().cloning()` or `unbatching().copying()`,
    /// consider using [`unbatching_ref()`](Collector::unbatching_ref) instead,
    /// which avoids unnecessary cloning.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    /// use std::ops::ControlFlow;
    ///
    /// let mut collector = vec![]
    ///     .into_collector()
    ///     .unbatching(|v, arr: &[_]| {
    ///         v.collect_many(arr.iter().copied());
    ///         ControlFlow::Continue(())
    ///     });
    ///
    /// assert!(collector.collect(&[1, 2, 3]).is_continue());
    /// assert!(collector.collect(&[4, 5]).is_continue());
    /// assert!(collector.collect(&[6, 7, 8, 9]).is_continue());
    ///
    /// assert_eq!(collector.finish(), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`combine`]: crate::collector::RefCollector::combine
    fn unbatching<F>(self, f: F) -> Unbatching<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut Self, T) -> ControlFlow<()>,
    {
        assert_collector::<_, T>(Unbatching::new(self, f))
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
