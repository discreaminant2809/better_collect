use super::{
    Chain, Cloning, Copying, Filter, Fuse, IntoCollector, Map, MapOutput, MapRef, Partition, Skip,
    Take, TakeWhile, Unbatching, UnbatchingRef, Unzip,
};
#[cfg(feature = "unstable")]
use super::{Nest, NestExact};

use crate::{assert_collector, assert_ref_collector};

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
pub trait Collector {
    /// Type of items this collector collects and accumulates.
    // Although it is tempting to put it in generic instead (since `String` can collect
    // `char` and `&str`, and `Count` can collect basically everything),
    // it will break type coherence because the compiler cannot decide which generic to use.
    // It turns out the "adaptor pattern" doesn't work well with generic traits.
    type Item;

    /// The result this collector yields, via the [`finish`](Collector::finish) method.
    ///
    /// This assosciated type does not appear in trait objects.
    type Output
    where
        Self: Sized;

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
    /// Some collectors-like [`take(0)`](Collector::take) and [`take_while()`]-only
    /// know when they are done after collecting an item, which might be too late
    /// if the item cannot be “afforded” and is lost forever.
    /// In this case, call [`break_hint()`](Collector::break_hint) before collecting
    /// (see its documentation to use it correctly).
    /// For "infinite" collectors (like most collections), this is not an issue
    /// since they can simply return  [`Continue(())`] every time.
    ///
    /// If the collector is uncertain - like "maybe I won’t accumulate… uh, fine, I will" -
    /// it is recommended to return [`Continue(())`].
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
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()>;

    /// Consumes the collector and returns the accumulated result.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let v = vec![1, 2, 3]
    ///     .into_collector()
    ///     .take(999)
    ///     .fuse()
    ///     .filter(|&x| x > 0);
    ///
    /// assert_eq!(v.finish(), [1, 2, 3]);
    /// ```
    fn finish(self) -> Self::Output
    where
        Self: Sized;

    /// Returns a hint whether the collector has stopped accumulating.
    ///
    /// Returns `true` if it is guaranteed that the collector has stopped accumulating,
    /// or returns `false` otherwise.
    ///
    /// As specified in the trait's documentation, after the stop is signaled somewhere else,
    /// including through [`collect()`](Collector::collect) or similar methods,
    /// or this method itself, the behavior of this method is unspecified.
    /// This may include returning `false` even if the collector has conceptually stopped.
    ///
    /// This method should be called once and only once before collecting
    /// items in a loop to avoid consuming one item prematurely.
    /// It is not intended for repeatedly checking whether the
    /// collector has stopped. Use [`fuse()`](Collector::fuse) if you find yourself
    /// needing such behavior.
    ///
    /// If the collector is uncertain, like "maybe I won’t accumulate… uh, fine, I will,"
    /// it is recommended to return `false`.
    /// For example, [`filter()`](Collector::filter) might skip some items it collects,
    /// but still returns `false` as long as the underlying collector can still accumulate.
    /// The filter just denies "undesirable" items, not signal termination
    /// (this is the job of [`take_while()`](Collector::take_while) instead).
    ///
    /// The default implementation always returns `false`.
    ///
    /// # Examples
    ///
    /// Correct usage:
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![]
    ///     .into_collector()
    ///     .take_while(|&x| x != 3);
    ///
    /// let mut has_stopped = collector.break_hint();
    /// let mut num = 0;
    /// while !has_stopped {
    ///     has_stopped = collector.collect(num).is_break();
    ///     num += 1;
    /// }
    ///
    /// assert_eq!(collector.finish(), [0, 1, 2]);
    /// ```
    ///
    /// Incorrect usage:
    ///
    /// ```no_run
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![]
    ///     .into_collector()
    ///     .take_while(|&x| x != 3);
    ///
    /// let mut num = 0;
    /// // If `collect()` has returned `Break(())` in the previous iteration,
    /// // The usage of `break_hint()` here is NOT valid. ⚠️
    /// // By the current implementation, this may loop indefinitely
    /// // until your RAM explodes! (the `Vec` keeps expanding)
    /// while !collector.break_hint() {
    ///     let _ = collector.collect(num);
    ///     num += 1;
    /// }
    ///
    /// // May not be correct anymore. ⚠️
    /// assert_eq!(collector.finish(), [0, 1, 2]);
    /// ```
    #[inline]
    fn break_hint(&self) -> bool {
        false
    }

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
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()>
    where
        Self: Sized,
    {
        if self.break_hint() {
            ControlFlow::Break(())
        } else {
            // Use `try_for_each` instead of `for` loop since the iterator may not be optimal for `for` loop
            // (e.g. `skip`, `chain`, etc.)
            items.into_iter().try_for_each(|item| self.collect(item))
        }
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
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output
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

    /// Creates a [`Collector`] that can "safely" collect items even after
    /// the underlying collector has stopped accumulating,
    /// without triggering undesired behaviors.
    ///
    /// Normally, a collector having stopped may behave unpredictably,
    /// including accumulating again.
    /// `fuse()` ensures that once a collector has stopped, subsequent items
    /// are guaranteed to **not** be accumulated. This means that at that point,
    /// the following are guaranteed on `fuse()`:
    ///
    /// - [`collect()`](Collector::collect) and similar methods always return
    ///   [`Break(())`].
    /// - [`break_hint()`](Collector::break_hint) always return `true`.
    ///
    /// This adaptor implements [`RefCollector`] if the underlying collector does.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// // `take_while()` is one of a few collectors that do NOT fuse internally.
    /// let mut collector = vec![].into_collector().take_while(|&x| x != 3);
    ///
    /// assert!(collector.collect(1).is_continue());
    /// assert!(collector.collect(2).is_continue());
    /// assert!(collector.collect(3).is_break());
    ///
    /// // Use after `Break` ⚠️
    /// let _ = collector.collect(4);
    ///
    /// // What do you think what `collector.finish()` would yield? You can try it yourself.
    /// // (Spoiler: by the current implementation, it may NOT be `[1, 2]`!)
    /// # // Not shown to the doc. We only confirm our claim here.
    /// # assert_ne!(collector.finish(), [1, 2]);
    ///
    /// // Now try `fuse()`.
    /// let mut collector = vec![].into_collector().take_while(|&x| x != 3).fuse();
    ///
    /// assert!(collector.collect(1).is_continue());
    /// assert!(collector.collect(2).is_continue());
    /// assert!(collector.collect(3).is_break());
    ///
    /// // From now on, there's only `Break`. No further items are accumulated.
    /// assert!(collector.collect(4).is_break());
    /// assert!(collector.collect(5).is_break());
    /// assert!(collector.collect_many([6, 7, 8, 9]).is_break());
    ///
    /// // The output is consistent again.
    /// assert_eq!(collector.finish(), [1, 2]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`Continue(())`]: ControlFlow::Continue
    /// [`Break(())`]: ControlFlow::Break
    #[inline]
    fn fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        assert_collector(Fuse::new(self))
    }

    /// Use [`cloning()`](Collector::cloning).
    #[inline]
    #[deprecated(since = "0.3.0", note = "Use `cloning()`")]
    fn cloned(self) -> Cloning<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        self.cloning()
    }

    /// Creates a [`RefCollector`] that [`clone`](Clone::clone)s every collected item.
    ///
    /// This is useful when you need ownership of items, but you still want to [`combine`]
    /// the underlying collector into another collector.
    /// (Reminder: only [`RefCollector`]s are [`combine`]-able.)
    ///
    /// You may not need this adaptor when working with [`Copy`] types (e.g., primitive types)
    /// since collectors usually implement [`RefCollector`] to collect them seamlessly.
    /// However, for non-[`Copy`] types like [`String`], this adaptor becomes necessary.
    ///
    /// As a [`Collector`], `cloning()` does nothing (effectively a no-op) and is usually useless
    /// at the end of a [`combine`] chain.
    /// It only performs its intended behavior when used as a [`RefCollector`].
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let collector_res = ["a", "b", "c"]
    ///     .into_iter()
    ///     .map(String::from)
    ///     // `Vec<String>` does not implement `RefCollector`,
    ///     // so we must call `cloning()` to make it `combine`-able.
    ///     // Otherwise, the first `Vec` would consume each item,
    ///     // leaving nothing for the second.
    ///     .feed_into(vec![].into_collector().cloning().combine(vec![]));
    ///
    /// let desired_vec = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
    /// assert_eq!(collector_res, (desired_vec.clone(), desired_vec));
    ///
    /// // Equivalent to:
    /// let unzip_res: (Vec<_>, Vec<_>) = ["a", "b", "c"]
    ///     .into_iter()
    ///     .map(String::from)
    ///     .map(|s| (s.clone(), s))
    ///     .unzip();
    ///
    /// assert_eq!(collector_res, unzip_res);
    /// ```
    ///
    /// For [`Copy`] types, this adaptor is usually unnecessary:
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let collector_res = [1, 2, 3]
    ///     .into_iter()
    ///     // Just `combine` normally.
    ///     // `Vec<i32>::IntoCollector` implements `RefCollector` since `i32` is `Copy`.
    ///     .feed_into(vec![].into_collector().combine(vec![]));
    ///
    /// assert_eq!(collector_res, (vec![1, 2, 3], vec![1, 2, 3]));
    ///
    /// // Equivalent to:
    /// let unzip_res: (Vec<_>, Vec<_>) = [1, 2, 3]
    ///     .into_iter()
    ///     .map(|num| (num, num))
    ///     .unzip();
    ///
    /// assert_eq!(collector_res, unzip_res);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`combine`]: crate::collector::RefCollector::combine
    #[inline]
    fn cloning(self) -> Cloning<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        assert_ref_collector(Cloning::new(self))
    }

    /// Use [`copying()`](Collector::copying).
    #[inline]
    #[deprecated(since = "0.3.0", note = "Use `copying()`")]
    fn copied(self) -> Copying<Self>
    where
        Self: Sized,
        Self::Item: Copy,
    {
        self.copying()
    }

    /// Creates a [`RefCollector`] that copies every collected item.
    ///
    /// This is useful when you need ownership of items, but you still want to [`combine`]
    /// the underlying collector into another collector.
    /// (Reminder: only [`RefCollector`]s are [`combine`]-able.)
    ///
    /// You usually don’t need this adaptor when working with [`Copy`] types (e.g., primitives),
    /// since collectors often implement [`RefCollector`] to collect them seamlessly.
    /// However, if your collector does not support it, this adaptor provides a fallback.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let collector_copying_res = [1, 2, 3]
    ///     .into_iter()
    ///     .feed_into(vec![].into_collector().copying().combine(vec![]));
    ///
    /// assert_eq!(collector_copying_res, (vec![1, 2, 3], vec![1, 2, 3]));
    ///
    /// // Equivalent to:
    /// let unzip_res: (Vec<_>, Vec<_>) = [1, 2, 3]
    ///     .into_iter()
    ///     .map(|s| (s, s))
    ///     .unzip();
    ///
    /// assert_eq!(collector_copying_res, unzip_res);
    ///
    /// // Also equivalent to using `combine` directly,
    /// // since `Vec<i32>::IntoCollector` implements `RefCollector`.
    /// let collector_normal_res = [1, 2, 3]
    ///     .into_iter()
    ///     .feed_into(vec![].into_collector().combine(vec![]));
    ///
    /// assert_eq!(collector_copying_res, collector_normal_res);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`combine`]: crate::collector::RefCollector::combine
    #[inline]
    fn copying(self) -> Copying<Self>
    where
        Self: Sized,
        Self::Item: Copy,
    {
        assert_ref_collector(Copying::new(self))
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
    ///         ConcatStr::new()
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
    fn map<F, T>(self, f: F) -> Map<Self, T, F>
    where
        Self: Sized,
        F: FnMut(T) -> Self::Item,
    {
        assert_collector(Map::new(self, f))
    }

    /// Creates a [`RefCollector`] that calls a closure on each item by mutable reference before collecting.
    ///
    /// This is used when the [`combine`](crate::collector::RefCollector::combine) chain expects to collect `T`,
    /// but you have a collector that collects `U`.
    /// In that case, you can use `map_ref()` to transform `T` into `U`.
    ///
    /// Unlike [`map()`](Collector::map), this adaptor only receives a mutable reference to each item.
    /// Because of that, it can be used in the middle of a [`combine`] chain,
    /// since it is a [`RefCollector`].
    /// While it can also appear at the end of the chain, consider using [`map()`](Collector::map) there
    /// instead for better clarity.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let (lens, _strings) = ["a".to_owned(), "bcd".to_owned(), "ef".to_owned()]
    ///     .into_iter()
    ///     .feed_into(
    ///         vec![]
    ///             .into_collector()
    ///             // Since we can only "view" the string via &mut,
    ///             // we use this adaptor to avoid cloning.
    ///             // (Limitation: type annotation may be required.)
    ///             .map_ref(|s: &mut String| s.len())
    ///             .combine(ConcatString::new())
    ///     );
    ///
    /// assert_eq!(lens, [1, 3, 2]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`combine`]: crate::collector::RefCollector::combine
    #[inline]
    fn map_ref<F, T>(self, f: F) -> MapRef<Self, T, F>
    where
        Self: Sized,
        F: FnMut(&mut T) -> Self::Item,
    {
        assert_ref_collector(MapRef::new(self, f))
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
        F: FnMut(&Self::Item) -> bool,
    {
        assert_collector(Filter::new(self, pred))
    }

    // fn modify()

    // fn filter_map()
    // fn filter_map_ref()

    // fn flat_map()

    /// Creates a [`Collector`] that stops accumulating after collecting the first `n` items,
    /// or fewer if the underlying collector ends sooner.
    ///
    /// `take(n)` collects items until either `n` items have been collected or the underlying collector
    /// stops, whichever happens first.
    /// For collections, the [`Output`](Collector::Output) will contain at most `n` more items than
    /// it had before construction.
    ///
    /// This also implements [`RefCollector`] if the underlying collector does.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![].into_collector().take(3);
    ///
    /// assert!(collector.collect(1).is_continue());
    /// assert!(collector.collect(2).is_continue());
    ///
    /// // Immediately stops after the third item.
    /// assert!(collector.collect(3).is_break());
    /// # // Internal assertion.
    /// # assert!(collector.collect(4).is_break());
    ///
    /// assert_eq!(collector.finish(), [1, 2, 3]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    #[inline]
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, n)
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
    /// let mut collector = ConcatStr::new().take_while(|&s| s != "stop");
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
        F: FnMut(&Self::Item) -> bool,
    {
        assert_collector(TakeWhile::new(self, pred))
    }

    /// Creates a [`Collector`] that skips the first `n` collected items before it begins
    /// accumulating them.
    ///
    /// `skip(n)` ignores collected items until `n` items have been collected. After that,
    /// subsequent items are accumulated normally.
    ///
    /// Note that in the current implementation,
    /// even if the underlying collector has stopped accumulating from the start,
    /// its [`collect()`] and similar methods will **not** return [`Break(())`], and
    /// [`break_hint()`] will **not** return `true` if it has not skipped enough items,
    /// and will still collect until the number of skipped items is `n`.
    ///
    /// This also implements [`RefCollector`] if the underlying collector does.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![].into_collector().skip(3);
    ///
    /// assert!(collector.collect(1).is_continue());
    /// assert!(collector.collect(2).is_continue());
    /// assert!(collector.collect(3).is_continue());
    ///
    /// // It has skipped enough items.
    /// assert!(collector.collect(4).is_continue());
    /// assert!(collector.collect(5).is_continue());
    ///
    /// assert_eq!(collector.finish(), [4, 5]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`Break(())`]: ControlFlow::Break
    /// [`collect()`]: Collector::collect
    /// [`break_hint()`]: Collector::break_hint
    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        assert_collector(Skip::new(self, n))
    }

    // fn step_by()

    /// Creates a [`Collector`] that feeds every item in the first collector until it stops accumulating,
    /// then continues feeding items into the second one.
    ///
    /// The first collector should be finite (typically achieved with [`take`](Collector::take)
    /// or [`take_while`](Collector::take_while)),
    /// otherwise it will hoard all incoming items and never pass any to the second.
    ///
    /// The [`Output`](Collector::Output) is a tuple containing the outputs of both underlying collectors,
    /// in order.
    ///
    /// This adaptor also implements [`RefCollector`] if both underlying collectors do.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![].into_collector().take(2).chain(vec![]);
    ///
    /// assert!(collector.collect(1).is_continue());
    ///
    /// // Now the first collector stops accumulating, but the second one is still active.
    /// assert!(collector.collect(2).is_continue());
    ///
    /// // Now the second one takes the spotlight.
    /// assert!(collector.collect(3).is_continue());
    /// assert!(collector.collect(4).is_continue());
    /// assert!(collector.collect(5).is_continue());
    ///
    /// assert_eq!(collector.finish(), (vec![1, 2], vec![3, 4, 5]));
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    #[inline]
    fn chain<C>(self, other: C) -> Chain<Self, C::IntoCollector>
    where
        Self: Sized,
        C: IntoCollector<Item = Self::Item>,
    {
        assert_collector(Chain::new(self, other.into_collector()))
    }

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
        C: IntoCollector<Item = Self::Item>,
        F: FnMut(&mut Self::Item) -> bool,
    {
        assert_collector(Partition::new(self, other_if_false.into_collector(), pred))
    }

    /// Creates a [`Collector`] that destructures each 2-tuple `(A, B)` item and distributes its fields:
    /// `A` goes to the first collector, and `B` goes to the second collector.
    ///
    /// `unzip()` is useful when you want to split an [`Iterator`]
    /// producing tuples or structs into multiple collections.
    ///
    /// This adaptor also implements [`RefCollector`] if both underlying collectors do.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// struct User {
    ///     id: u32,
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let users = [
    ///     User {
    ///         id: 1,
    ///         name: "Alice".to_owned(),
    ///         email: "alice@mail.com".to_owned(),
    ///     },
    ///     User {
    ///         id: 2,
    ///         name: "Bob".to_owned(),
    ///         email: "bob@mail.com".to_owned(),
    ///     },
    /// ];
    ///
    /// let ((ids, names), emails) = users
    ///     .into_iter()
    ///     .feed_into(
    ///         vec![]
    ///             .into_collector()
    ///             .unzip(vec![])
    ///             .unzip(vec![])
    ///             .map(|user: User| ((user.id, user.name), user.email)),
    ///     );
    ///
    /// assert_eq!(ids, [1, 2]);
    /// assert_eq!(names, vec!["Alice", "Bob"]);
    /// assert_eq!(emails, vec!["alice@mail.com", "bob@mail.com"]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    #[inline]
    fn unzip<C>(self, other: C) -> Unzip<Self, C::IntoCollector>
    where
        Self: Sized,
        C: IntoCollector,
    {
        assert_collector(Unzip::new(self, other.into_collector()))
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
    fn unbatching<T, F>(self, f: F) -> Unbatching<Self, T, F>
    where
        Self: Sized,
        F: FnMut(&mut Self, T) -> ControlFlow<()>,
    {
        assert_collector(Unbatching::new(self, f))
    }

    /// Creates a [`RefCollector`] with a custom collection logic.
    ///
    /// This adaptor is useful for behaviors that cannot be expressed
    /// through existing adaptors without cloning or intermediate allocations.
    ///
    /// Unlike [`unbatching()`](Collector::unbatching), this adaptor only receives
    /// a mutable reference to each item.
    /// Because of that, it can be used in the middle of a [`combine`] chain,
    /// since it is a [`RefCollector`].
    /// While it can also appear at the end of the chain, consider using
    /// [`unbatching()`](Collector::unbatching) there instead for better clarity.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{prelude::*, collector::Sink};
    /// use std::ops::ControlFlow;
    ///
    /// let matrix = vec![
    ///     vec![1, 2, 3],
    ///     vec![4, 5, 6],
    ///     vec![7, 8, 9],
    /// ];
    ///
    /// let (flattened, _) = matrix
    ///     .into_iter()
    ///     .feed_into(
    ///         vec![]
    ///             .into_collector()
    ///             .unbatching_ref(|v, row: &mut Vec<_>| {
    ///                 v.collect_many(row.iter().copied());
    ///                 ControlFlow::Continue(())
    ///             })
    ///             .combine(Sink::new())
    ///     );
    ///
    /// assert_eq!(flattened, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`combine`]: crate::collector::RefCollector::combine
    fn unbatching_ref<T, F>(self, f: F) -> UnbatchingRef<Self, T, F>
    where
        Self: Sized,
        F: FnMut(&mut Self, &mut T) -> ControlFlow<()>,
    {
        assert_ref_collector(UnbatchingRef::new(self, f))
    }

    /// Creates a [`Collector`] that transforms the final accumulated result.
    ///
    /// This is used when your output gets "ugly" after a chain of adaptors,
    /// or when you do not want to break your API by (accidentally) rearranging adaptors,
    /// or when you just want a different output type for your collector.
    ///
    /// This also implements [`RefCollector`] if the underlying collector does.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{prelude::*, num::Sum, cmp::Max};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Stats {
    ///     sum: i32,
    ///     max: i32,
    /// }
    ///
    /// let mut collector = Sum::<i32>::new()
    ///     .combine(Max::new())
    ///     .map_output(|(sum, max)| Stats { sum, max: max.unwrap() });
    ///
    /// assert!(collector.collect(1).is_continue());
    /// assert!(collector.collect(3).is_continue());
    /// assert!(collector.collect(2).is_continue());
    ///
    /// assert_eq!(collector.finish(), Stats { sum: 6, max: 3 });
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    fn map_output<T, F>(self, f: F) -> MapOutput<Self, T, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> T,
    {
        assert_collector(MapOutput::new(self, f))
    }

    /// Creates a [`Collector`] that collects all outputs produced by an inner collector.
    ///
    /// The inner collector collects items first until it stops accumulating,
    /// then, the outer collector collects the output produced by the inner collector,
    /// then repeat.
    ///
    /// The inner collector must implement [`Clone`]. Also, it should be finite
    /// so that the outer can collect more, or else the outer will be stuck with
    /// one output forever.
    ///
    /// This version collects the unfinished inner (the remainder), if any,
    /// after calling [`finish()`] or [`collect_then_finish()`].
    /// Hence, this adaptor is not "exact," similar to [`[_]::chunks()`](slice::chunks).
    /// Use [`nest_exact()`](Collector::nest_exact) if you do not care about the remainder,
    /// since the exact verion is generally faster.
    ///
    /// This also implements [`RefCollector`] if the inner collector does.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![]
    ///     .into_collector()
    ///     .nest(vec![].into_collector().take(3));
    ///
    /// assert!(collector.collect_many(1..=11).is_continue());
    ///
    /// assert_eq!(
    ///     collector.finish(),
    ///     [
    ///         vec![1, 2, 3],
    ///         vec![4, 5, 6],
    ///         vec![7, 8, 9],
    ///         vec![10, 11],
    ///     ],
    /// );
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`finish()`]: Collector::finish
    /// [`collect_then_finish()`]: Collector::collect_then_finish
    #[cfg(feature = "unstable")]
    fn nest<C>(self, inner: C) -> Nest<Self, C::IntoCollector>
    where
        Self: Collector<Item = C::Output> + Sized,
        C: IntoCollector<IntoCollector: Clone>,
    {
        assert_collector(Nest::new(self, inner.into_collector()))
    }

    /// Creates a [`Collector`] that collects all outputs produced by an inner collector.
    ///
    /// The inner collector collects items first until it stops accumulating,
    /// then, the outer collector collects the output produced by the inner collector,
    /// then repeat.
    ///
    /// The inner collector must implement [`Clone`]. Also, it should be finite
    /// so that the outer can collect more, or else the outer will be stuck with
    /// one output forever.
    ///
    /// This version will only collect all the inners that has stopped accumulating.
    /// Any unfinished inner (the remainder) is discarded after calling
    /// [`finish()`] or [`collect_then_finish()`].
    /// Hence, this adaptor is "exact," similar to [`[_]::chunks_exact()`](slice::chunks_exact).
    /// Since the implementation is simpler, this adaptor is generally faster.
    /// Use [`nest()`](Collector::nest) if you care about the remainder.
    ///
    /// This also implements [`RefCollector`] if the inner collector does.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![]
    ///     .into_collector()
    ///     .nest_exact(vec![].into_collector().take(3));
    ///
    /// assert!(collector.collect_many(1..=11).is_continue());
    ///
    /// assert_eq!(
    ///     collector.finish(),
    ///     [
    ///         [1, 2, 3],
    ///         [4, 5, 6],
    ///         [7, 8, 9],
    ///     ],
    /// );
    /// ```
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`finish()`]: Collector::finish
    /// [`collect_then_finish()`]: Collector::collect_then_finish
    #[cfg(feature = "unstable")]
    fn nest_exact<C>(self, inner: C) -> NestExact<Self, C::IntoCollector>
    where
        Self: Collector<Item = C::Output> + Sized,
        C: IntoCollector<IntoCollector: Clone>,
    {
        assert_collector(NestExact::new(self, inner.into_collector()))
    }
}

/// A mutable reference to a collect produce nothing.
///
/// This is useful when you *just* want to feed items to a collector without
/// finishing it.
impl<C> Collector for &mut C
where
    C: Collector,
{
    type Item = C::Item;

    type Output = ();

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        C::collect(self, item)
    }

    #[inline]
    fn finish(self) -> Self::Output {}

    #[inline]
    fn break_hint(&self) -> bool {
        C::break_hint(self)
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // FIXED: specialization for unsized type.
        // We can't add `?Sized` to the bound of `C` because this method requires `Sized`.
        C::collect_many(self, items)
    }

    // The default implementation for `collect_then_finish()` is sufficient.
}

macro_rules! dyn_impl {
    ($($traits:ident)*) => {
        impl<'a, T> Collector for &mut (dyn Collector<Item = T> $(+ $traits)* + 'a) {
            type Item = T;

            type Output = ();

            #[inline]
            fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
                <dyn Collector<Item = T>>::collect(*self, item)
            }

            #[inline]
            fn finish(self) -> Self::Output {}

            #[inline]
            fn break_hint(&self) -> bool {
                <dyn Collector<Item = T>>::break_hint(self)
            }

            // The default implementation are sufficient.
        }
    };
}

dyn_impl!();
dyn_impl!(Send);
dyn_impl!(Sync);
dyn_impl!(Send Sync);

// `Output` shouldn't be required to ne specified.
fn _dyn_compatible<T>(_: &mut dyn Collector<Item = T>) {}

#[cfg(test)]
mod tests {
    use crate::{collector::Sink, prelude::*};

    #[test]
    fn break_hint_needed() {
        let mut iter = [(); 3].into_iter();
        Sink::new()
            .take(0)
            .combine(())
            .combine(())
            .collect_then_finish(&mut iter);

        assert_eq!(iter.count(), 3);
    }
}
