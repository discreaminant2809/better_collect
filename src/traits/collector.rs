use std::ops::ControlFlow;

use crate::{
    Chain, Cloned, Copied, Filter, Fuse, Map, MapRef, Partition, Take, Unzip, assert_collector,
    assert_ref_collector,
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
/// # Example
///
/// Suppose we’re building a tokenizer to process text for an NLP model.
/// We’ll skip all complicated details for now and simply collect every word we see.
///
/// ```
/// use std::{ops::ControlFlow, collections::HashMap};
/// use better_collect::{Collector, BetterCollect};
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
/// impl Collector<String> for Tokenizer {
///     // Usually, the collector itself is also the final result.
///     type Output = Self;
///
///     fn collect(&mut self, word: String) -> ControlFlow<()> {
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
/// let sentence = "the nobel and the singer";
/// let tokenizer = sentence
///     .split_whitespace()
///     .map(String::from)
///     .better_collect(Tokenizer::default());
///
/// // "the" should only appear once.
/// assert_eq!(tokenizer.words, ["the", "nobel", "and", "singer"]);
/// assert_eq!(tokenizer.tokenize("the singer and the swordswoman"), [1, 4, 3, 1, 0]);
/// ```
pub trait Collector<T>: Sized {
    /// The result this collector yields, via the [`finish`](Collector::finish) method.
    type Output;

    /// Collects an item and returns a [`ControlFlow`] indicating whether the collector is “closed”
    /// — meaning it will no longer accumulate items **right after** this operation.
    ///
    /// Return [`Continue(())`] to indicate the collector can still accumulate more items,
    /// or [`Break(())`] if it will no longer accumulate from now on and further feeding is meaningless.
    ///
    /// This is analogous to [`Iterator::next`], which returns an item (instead of collecting one)
    /// and signals with [`None`] whenever it finishes.
    ///
    /// Implementors should return this hint carefully and inform the caller the closure
    /// as early as possible. This can usually be upheld, but not always.
    /// Some collectors-like [`take(0)`](Collector::take) and `take_while()`-only
    /// know when they are done after collecting an item, which might be too late
    /// if the item cannot be “afforded” and is lost forever.
    /// For "infinite" collectors (like most collections), this is not an issue
    /// since they can simply return  [`Continue(())`] every time.
    ///
    /// It is also allowed for a collector to be "reopened" later and resume accumulating
    /// items normally. (Just like [`Iterator::next`] might start yielding again).
    /// That is why the returned [`ControlFlow`] is only a hint -
    /// this allows optimization (e.g. no need for an internal flag).
    /// To prevent a collector from resuming, wrap it with [`fuse()`](Collector::fuse).
    ///
    /// If the collector is uncertain - like "maybe I won’t accumulate… uh, fine, I will" -
    /// it is recommended to return [`Continue(())`].
    /// For example, [`filter()`](Collector::filter) might skip some items it collects,
    /// but still returns [`Continue(())`] as long as the underlying collector can still accumulate;
    /// The filter just denies "undesirable" items, not signal termination
    /// (this is the job of `take_while()` instead).
    ///
    /// Collectors with limited capacity (e.g., a `Vec` stored on the stack) will eventually
    /// return [`Break(())`] once full, right after the last item is accumulated.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{Collector, Last};
    ///
    /// let mut collector = vec![].take(3); // only takes 3 items
    ///
    /// // It has not reached its 3-item quota yet.
    /// assert!(collector.collect(1).is_continue());
    /// assert!(collector.collect(2).is_continue());
    ///
    /// // After collecting `3`, it meets the quota, so it signals `Break` immediately.
    /// assert!(collector.collect(3).is_break());
    ///
    /// // Further feeding does nothing.
    /// assert!(collector.collect(4).is_break());
    ///
    /// assert_eq!(collector.finish(), [1, 2, 3]);
    ///
    /// // Most collectors can accumulate indefinitely.
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
    fn collect(&mut self, item: T) -> ControlFlow<()>;

    /// Consumes the collector and returns the accumulated result.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::Collector;
    ///
    /// let v = vec![1, 2, 3]
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
    /// This method can be overridden for optimization.
    /// Implementors may choose a more efficient way to consume an iterator than a simple `for` loop
    /// ([`Iterator`] offers many alternative consumption methods), depending on the collector’s needs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use better_collect::Collector;
    ///
    /// let mut v = vec![1, 2];
    /// v.collect_many([3, 4, 5]);
    ///
    /// assert_eq!(v, [1, 2, 3, 4, 5]);
    /// ```
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
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
    /// use better_collect::Collector;
    ///
    /// let mut v = vec![1, 2];
    ///
    /// assert_eq!(v.collect_then_finish([3, 4, 5]), [1, 2, 3, 4, 5]);
    /// ```
    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        // Do this instead of putting `mut` in `self` since some IDEs are stupid
        // and just put `mut self` in every generated code.
        let mut this = self;

        // We don't care whether the collector breaks or not, since if it doesn't it'll have
        // completely depleted the iterator so... we just finish--nothing changed.
        let _ = this.collect_many(items);
        this.finish()
    }

    /// Creates a collector that stops accumulating after the first [`Break(())`].
    ///
    /// After a collector returns [`Break(())`], future calls may or may not return [`Continue(())`] again.
    /// `fuse()` ensures that after [`Break(())`] is returned, it will always return [`Break(())`] forever.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{BetterCollect, Collector, Count};
    /// use std::ops::ControlFlow;
    ///
    /// // A collector that alternates between `Continue` and `Break`.
    /// #[derive(Default)]
    /// struct NastyCollector {
    ///     is_continue: bool,
    /// }
    ///
    /// impl Collector<()> for NastyCollector {
    ///     type Output = ();
    ///
    ///     fn collect(&mut self, _: ()) -> ControlFlow<()> {
    ///         self.is_continue = !self.is_continue;
    ///         
    ///         if self.is_continue {
    ///             ControlFlow::Continue(())
    ///         } else {
    ///             ControlFlow::Break(())
    ///         }
    ///     }
    ///
    ///     fn finish(self) -> Self::Output {}
    /// }
    ///
    /// let mut collector = NastyCollector::default();
    ///
    /// // It signals "nastily"
    /// assert!(collector.collect(()).is_continue());
    /// assert!(collector.collect(()).is_break());
    /// assert!(collector.collect(()).is_continue());
    /// assert!(collector.collect(()).is_break());
    ///
    /// // We try the fused version
    /// let mut collector = NastyCollector::default().fuse();
    ///
    /// assert!(collector.collect(()).is_continue());
    /// assert!(collector.collect(()).is_break());
    ///
    /// // Now the hint is stably `Break`
    /// assert!(collector.collect(()).is_break());
    /// assert!(collector.collect(()).is_break());
    /// assert!(collector.collect(()).is_break());
    /// ```
    ///
    /// [`Continue(())`]: ControlFlow::Continue
    /// [`Break(())`]: ControlFlow::Break
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

    #[inline]
    fn copied(self) -> Copied<Self>
    where
        T: Copy,
    {
        assert_ref_collector(Copied::new(self))
    }

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
    fn chain<C>(self, other: C) -> Chain<Self, C>
    where
        C: Collector<T>,
    {
        assert_collector(Chain::new(self, other))
    }

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
