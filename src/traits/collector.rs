use std::ops::ControlFlow;

use crate::{
    Chain, Cloned, Copied, Filter, Fuse, IntoCollector, Map, MapRef, Partition, Skip, Take,
    TakeWhile, Unbatching, UnbatchingRef, Unzip, assert_collector, assert_ref_collector,
};

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
/// Unless stated otherwise by the collector’s implementation, the behavior of
/// [`Collector::collect()`], [`Collector::collect_many()`], and
/// [`RefCollector::collect_ref()`](crate::RefCollector::collect_ref)
/// **after** any of them have returned [`Break(())`] is unspecified.
///
/// After that point, subsequent calls to **any** method other than [`finish()`](Collector::finish)
/// may behave arbitrarily. They may panic, overflow, or even resume accumulation
/// (similar to how [`Iterator::next()`] might yield again after returning [`None`]).
/// Callers should generally call [`finish()`](Collector::finish) once a collector
/// returns [`Break(())`].
/// If this invariant cannot be upheld, wrap it with [`fuse()`](Collector::fuse).
///
/// This looseness allows for optimizations (for example, omitting an internal “closed” flag).
///
/// Although the behavior is unspecified, this method is **not** `unsafe`.
/// Implementors **must not** cause memory corruption, undefined behavior,
/// or any other safety violations — and callers **must not** rely on such outcomes.
///
/// # Limitations
///
/// In some cases, you may need to explicitly annotate the parameter types in closures,
/// especially for adaptors that take generic functions.
/// This is due to current limitations in Rust’s type inference for closure parameters.
///
/// # Example
///
/// Suppose we’re building a tokenizer to process text for an NLP model.
/// We’ll skip all complicated details for now and simply collect every word we see.
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
///     // Usually, the collector itself is also the final result.
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
///     .better_collect(Tokenizer::default());
///
/// // "the" should only appear once.
/// assert_eq!(tokenizer.words, ["the", "noble", "and", "singer"]);
/// assert_eq!(tokenizer.tokenize("the singer and the swordswoman"), [1, 4, 3, 1, 0]);
/// ```
///
/// [`Break(())`]: std::ops::ControlFlow::Break
pub trait Collector: Sized {
    /// Type of items this collector collects and accumulates.
    // Although it is tempting to put it in generic instead (since `String` can collect
    // `char` and `&str`, and `Count` can collect basically everything),
    // it will break type coherence because the compiler cannot decide which generic to use.
    // It turns out the "adaptor pattern" doesn't work well with generic traits.
    type Item;

    /// The result this collector yields, via the [`finish`](Collector::finish) method.
    type Output;

    /// Collects an item and returns a [`ControlFlow`] indicating whether the collector is “closed”
    /// — meaning it will no longer accumulate items **right after** this operation.
    ///
    /// Return [`Continue(())`] to indicate the collector can still accumulate more items,
    /// or [`Break(())`] if it will no longer accumulate from now on and further feeding is meaningless.
    ///
    /// This is analogous to [`Iterator::next()`], which returns an item (instead of collecting one)
    /// and signals with [`None`] whenever it finishes.
    ///
    /// Implementors should return this hint carefully and inform the caller the closure
    /// as early as possible. This can usually be upheld, but not always.
    /// Some collectors-like [`take(0)`](Collector::take) and [`take_while()`]-only
    /// know when they are done after collecting an item, which might be too late
    /// if the item cannot be “afforded” and is lost forever.
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
    /// use better_collect::{prelude::*, Last};
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
    fn finish(self) -> Self::Output;

    /// Collects items from an iterator and returns a [`ControlFlow`] indicating whether the collector is “closed”
    /// — meaning it will no longer accumulate items **right after** the last possible item is collected,
    /// possibly none are collected.
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
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
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
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        // Do this instead of putting `mut` in `self` since some IDEs are stupid
        // and just put `mut self` in every generated code.
        let mut this = self;

        // We don't care whether the collector breaks or not, since if it doesn't it'll have
        // completely depleted the iterator so... we just finish--nothing changed.
        let _ = this.collect_many(items);
        this.finish()
    }

    /// Creates a [`Collector`] that stops accumulating permanently after the first [`Break(())`].
    ///
    /// Normally, a collector that returns [`Break(())`] may behave unpredictably,
    /// inclluding returning [`Continue(())`] again.
    /// `fuse()` ensures that once [`Break(())`] has been returned, it will **always**
    /// return [`Break(())`] forever, and subsequent items will **not** be accumulated.
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
    /// [`RefCollector`]: crate::RefCollector
    /// [`Continue(())`]: ControlFlow::Continue
    /// [`Break(())`]: ControlFlow::Break
    #[inline]
    fn fuse(self) -> Fuse<Self> {
        assert_collector(Fuse::new(self))
    }

    /// Creates a [`RefCollector`] that [`clone`](Clone::clone)s every collected item.
    ///
    /// This is useful when you need ownership of items, but you still want to [`then`]
    /// the underlying collector into another collector.
    /// (Reminder: only [`RefCollector`]s are [`then`]-able.)
    ///
    /// You may not need this adaptor when working with [`Copy`] types (e.g., primitive types)
    /// since collectors usually implement [`RefCollector`] to collect them seamlessly.
    /// However, for non-[`Copy`] types like [`String`], this adaptor becomes necessary.
    ///
    /// As a [`Collector`], `cloned()` does nothing (effectively a no-op) and is usually useless
    /// at the end of a [`then`] chain.
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
    ///     // so we must call `cloned()` to make it `then`-able.
    ///     // Otherwise, the first `Vec` would consume each item,
    ///     // leaving nothing for the second.
    ///     .better_collect(vec![].into_collector().cloned().then(vec![]));
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
    ///     // Just `then` normally.
    ///     // `Vec<i32>` implements `RefCollector` since `i32` is `Copy`.
    ///     .better_collect(vec![].into_collector().then(vec![]));
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
    /// [`RefCollector`]: crate::RefCollector
    /// [`then`]: crate::RefCollector::then
    #[inline]
    fn cloned(self) -> Cloned<Self>
    where
        Self::Item: Clone,
    {
        assert_ref_collector(Cloned::new(self))
    }

    /// Creates a [`RefCollector`] that copies every collected item.
    ///
    /// This is useful when you need ownership of items, but you still want to [`then`]
    /// the underlying collector into another collector.
    /// (Reminder: only [`RefCollector`]s are [`then`]-able.)
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
    /// let collector_copied_res = [1, 2, 3]
    ///     .into_iter()
    ///     .better_collect(vec![].into_collector().copied().then(vec![]));
    ///
    /// assert_eq!(collector_copied_res, (vec![1, 2, 3], vec![1, 2, 3]));
    ///
    /// // Equivalent to:
    /// let unzip_res: (Vec<_>, Vec<_>) = [1, 2, 3]
    ///     .into_iter()
    ///     .map(|s| (s, s))
    ///     .unzip();
    ///
    /// assert_eq!(collector_copied_res, unzip_res);
    ///
    /// // Also equivalent to using `then` directly, since `Vec<i32>` implements `RefCollector`.
    /// let collector_normal_res = [1, 2, 3]
    ///     .into_iter()
    ///     .better_collect(vec![].into_collector().then(vec![]));
    ///
    /// assert_eq!(collector_copied_res, collector_normal_res);
    /// ```
    ///
    /// [`RefCollector`]: crate::RefCollector
    /// [`then`]: crate::RefCollector::then
    #[inline]
    fn copied(self) -> Copied<Self>
    where
        Self::Item: Copy,
    {
        assert_ref_collector(Copied::new(self))
    }

    /// Creates a [`Collector`] that calls a closure on each item before collecting.
    ///
    /// This is used when a [`then`] chain expects to collect `T`,
    /// but you have a collector that collects `U`. In that case,
    /// you can use `map()` to transform `U` into `T` before passing it along.
    ///
    /// Since it does **not** implement [`RefCollector`], this adaptor should be used
    /// on the **final collector** in a [`then`] chain, or adapted into a [`RefCollector`]
    /// using the appropriate adaptor.
    /// If you find yourself writing `map().cloned()` or `map().copied()`,
    /// consider using [`map_ref()`](Collector::map_ref) instead, which avoids unnecessary cloning.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// // Collect the first 5 squared numbers.
    /// let collector_squares = (1..=5)
    ///     .better_collect(vec![].into_collector().map(|num| num * num));
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
    ///     .better_collect(
    ///         ConcatStr::new()
    ///             // Limitation: type annotation may be needed.
    ///             .then(vec![].into_collector().map(|s: &str| s.len()))
    ///     );
    ///
    /// assert_eq!(lens, [1, 3, 2]);
    /// ```
    ///
    /// [`RefCollector`]: crate::RefCollector
    /// [`then`]: crate::RefCollector::then
    #[inline]
    fn map<F, T>(self, f: F) -> Map<Self, T, F>
    where
        F: FnMut(T) -> Self::Item,
    {
        assert_collector(Map::new(self, f))
    }

    /// Creates a [`RefCollector`] that calls a closure on each item by mutable reference before collecting.
    ///
    /// This is used when the [`then`](crate::RefCollector::then) chain expects to collect `T`,
    /// but you have a collector that collects `U`.
    /// In that case, you can use `map_ref()` to transform `T` into `U`.
    ///
    /// Unlike [`map()`](Collector::map), this adaptor only receives a mutable reference to each item.
    /// Because of that, it can be used **in the middle** of a [`then`] chain,
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
    ///     .better_collect(
    ///         vec![]
    ///             .into_collector()
    ///             // Since we can only "view" the string via &mut,
    ///             // we use this adaptor to avoid cloning.
    ///             // (Limitation: type annotation may be required.)
    ///             .map_ref(|s: &mut String| s.len())
    ///             .then(ConcatString::new())
    ///     );
    ///
    /// assert_eq!(lens, [1, 3, 2]);
    /// ```
    ///
    /// [`RefCollector`]: crate::RefCollector
    /// [`then`]: crate::RefCollector::then
    #[inline]
    fn map_ref<F, T>(self, f: F) -> MapRef<Self, T, F>
    where
        F: FnMut(&mut T) -> Self::Item,
    {
        assert_ref_collector(MapRef::new(self, f))
    }

    /// Creates a [`Collector`] that uses a closure to determine whether an item should be collected.
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
    ///     .better_collect(
    ///         vec![]
    ///             .into_collector()
    ///             .filter(|&x| x % 2 == 0)
    ///             // Since `Vec<i32>` implements `RefCollector`,
    ///             // this adaptor does too!
    ///             .then(vec![].into_collector().filter(|&x| x < 0))
    ///     );
    ///
    /// assert_eq!(evens, [-4, -2, 0, 2, 4]);
    /// assert_eq!(negs, [-5, -4, -3, -2, -1]);
    /// ```
    ///
    /// [`RefCollector`]: crate::RefCollector
    /// [`Continue`]: std::ops::ControlFlow::Continue
    /// [`Break`]: std::ops::ControlFlow::Break
    #[inline]
    fn filter<F>(self, pred: F) -> Filter<Self, F>
    where
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
    /// stops - whichever happens first.
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
    /// [`RefCollector`]: crate::RefCollector
    #[inline]
    fn take(self, n: usize) -> Take<Self> {
        Take::new(self, n)
    }

    /// Creates a [`Collector`] that accumulates items as long as a predicate returns `true`.
    ///
    /// `take_while()` collects items until it encounters one for which the predicate returns `false`.
    /// That item—and all subsequent ones—will **not** be accumulated.
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
    /// [`RefCollector`]: crate::RefCollector
    fn take_while<F>(self, pred: F) -> TakeWhile<Self, F>
    where
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
    /// [`RefCollector`]: crate::RefCollector
    fn skip(self, n: usize) -> Skip<Self> {
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
    /// [`RefCollector`]: crate::RefCollector
    #[inline]
    fn chain<C>(self, other: C) -> Chain<Self, C::IntoCollector>
    where
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
    /// [`RefCollector`]: crate::RefCollector
    #[inline]
    fn partition<C, F>(self, pred: F, other_if_false: C) -> Partition<Self, C::IntoCollector, F>
    where
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
    ///     .better_collect(
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
    /// [`RefCollector`]: crate::RefCollector
    #[inline]
    fn unzip<C>(self, other: C) -> Unzip<Self, C::IntoCollector>
    where
        C: IntoCollector,
    {
        assert_collector(Unzip::new(self, other.into_collector()))
    }

    /// Creates a [`Collector`] with a custom collection logic.
    ///
    /// This adaptor is useful for behaviors that cannot be expressed
    /// through existing adaptors without cloning or intermediate allocations.
    ///
    /// Since it does **not** implement [`RefCollector`], this adaptor should be used
    /// on the **final collector** in a [`then`] chain, or adapted into a [`RefCollector`]
    /// using the appropriate adaptor.
    /// If you find yourself writing `unbatching().cloned()` or `unbatching().copied()`,
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
    /// [`RefCollector`]: crate::RefCollector
    /// [`then`]: crate::RefCollector::then
    fn unbatching<T, F>(self, f: F) -> Unbatching<Self, T, F>
    where
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
    /// Because of that, it can be used **in the middle** of a [`then`] chain,
    /// since it is a [`RefCollector`].
    /// While it can also appear at the end of the chain, consider using
    /// [`unbatching()`](Collector::unbatching) there instead for better clarity.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{prelude::*, Sink};
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
    ///     .better_collect(
    ///         vec![]
    ///             .into_collector()
    ///             .unbatching_ref(|v, row: &mut Vec<_>| {
    ///                 v.collect_many(row.iter().copied());
    ///                 ControlFlow::Continue(())
    ///             })
    ///             .then(Sink::new())
    ///     );
    ///
    /// assert_eq!(flattened, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    ///
    /// [`RefCollector`]: crate::RefCollector
    /// [`then`]: crate::RefCollector::then
    fn unbatching_ref<T, F>(self, f: F) -> UnbatchingRef<Self, T, F>
    where
        F: FnMut(&mut Self, &mut T) -> ControlFlow<()>,
    {
        assert_ref_collector(UnbatchingRef::new(self, f))
    }
}

/// A mutable reference to a collect produce nothing.
///
/// This is useful when you *just* want to feed items to a collector without
/// finishing it.
impl<C: Collector> Collector for &mut C {
    type Item = C::Item;

    type Output = ();

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        C::collect(self, item)
    }

    #[inline]
    fn finish(self) -> Self::Output {}

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        C::collect_many(self, items)
    }

    // The default implementation for `collect_then_finish()` is sufficient.
}
