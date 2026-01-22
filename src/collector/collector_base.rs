use std::ops::ControlFlow;

use super::{Cloning, Copying, Fuse, IntoCollector, MapOutput, Skip, Take, Unzip};

///
pub trait CollectorBase {
    /// The result this collector yields, via the [`finish()`](CollectorBase::finish) method.
    ///
    /// This assosciated type does not appear in trait objects.
    type Output
    where
        Self: Sized;

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
    /// it is recommended to just return `false`.
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
    fn break_hint(&self) -> ControlFlow<()> {
        ControlFlow::Continue(())
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
        assert_collector_base(Fuse::new(self))
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
    {
        Cloning::new(self)
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
    {
        Copying::new(self)
    }

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

    /// Creates a [`Collector`] that skips the first `n` collected items before it begins
    /// accumulating them.
    ///
    /// `skip(n)` ignores collected items until `n` items have been collected. After that,
    /// subsequent items are accumulated normally.
    ///
    /// Note that in the current implementation,
    /// if the underlying collector has stopped accumulating during skipping,
    /// its [`collect()`] and similar methods will return [`Break(())`] and
    /// [`break_hint()`] will return `true`,
    /// regardless of whether the adaptor has skipped enough items or not.
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
        Skip::new(self, n)
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
        assert_collector_base(Unzip::new(self, other.into_collector()))
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
    fn map_output<F, T>(self, f: F) -> MapOutput<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> T,
    {
        assert_collector_base(MapOutput::new(self, f))
    }
}

impl<C> CollectorBase for &mut C
where
    C: CollectorBase,
{
    type Output = ();

    fn finish(self) -> Self::Output {}

    fn break_hint(&self) -> ControlFlow<()> {
        C::break_hint(self)
    }
}

macro_rules! dyn_impl {
    ($($traits:ident)*) => {
        impl<'a> CollectorBase for &mut (dyn CollectorBase $(+ $traits)* + 'a) {
            type Output = ();

            #[inline]
            fn finish(self) -> Self::Output {}

            #[inline]
            fn break_hint(&self) -> ControlFlow<()> {
                <dyn CollectorBase>::break_hint(self)
            }
        }
    };
}

dyn_impl!();
dyn_impl!(Send);
dyn_impl!(Sync);
dyn_impl!(Send Sync);

// `Output` shouldn't be required to be specified.
fn _dyn_compatible(_: &mut dyn CollectorBase) {}

#[inline(always)]
fn assert_collector_base<C>(collector: C) -> C
where
    C: CollectorBase,
{
    collector
}
