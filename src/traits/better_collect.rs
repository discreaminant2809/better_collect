use crate::Collector;

#[cfg(feature = "unstable")]
use crate::{Driver, RefCollector, assert_iterator};

/// Extends [`Iterator`] with the [`better_collect`](BetterCollect::better_collect) method
/// for working seamlessly with [`Collector`]s.
///
/// This trait is automatically implemented for all [`Iterator`] types.
pub trait BetterCollect: Iterator {
    /// Extracts items from this iterator into the provided collector till
    /// the collector stops accumulating or the iterator is exhausted.
    /// and returns the collector’s output.
    ///
    /// To use this method, import the [`BetterCollect`] trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{
    ///     BetterCollect, Collector, RefCollector, IntoCollector,
    ///     cmp::Max,
    /// };
    ///
    /// let (nums, max) = [4, 2, 6, 3]
    ///     .into_iter()
    ///     .better_collect(vec![].into_collector().then(Max::new()));
    ///
    /// assert_eq!(nums, [4, 2, 6, 3]);
    /// assert_eq!(max, Some(6));
    /// ```
    #[inline]
    fn better_collect<C>(&mut self, collector: C) -> C::Output
    where
        C: Collector<Item = Self::Item>,
    {
        collector.collect_then_finish(self)
    }

    /// Extracts items from this iterator into the provided collector **as far as the
    /// puller drives the iterator**, then returns both the collector’s output and
    /// the puller’s result.
    ///
    /// The `puller` is a closure that receives an [`Iterator`] as a *driver*
    /// and produces an additional result.
    /// An item is collected **only when the driver advances pass that item**.
    /// If the driver is not fully exhausted, the iterator will not be fully
    /// collected either.
    ///
    /// Be careful when using short-circuiting methods on [`Iterator`] such as
    /// [`try_fold()`](Iterator::try_fold) or [`any()`](Iterator::any).
    /// They stop when something is satisfied, preventing the collector
    /// from collecting every item.
    /// Consider `for_each(drop)` the iterator before returning.
    /// if you want to exhaust the iterator.
    ///
    /// To use this method, import the [`BetterCollect`] trait.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use better_collect::{BetterCollect, string::ConcatStr};
    ///
    /// let (s_no_ws, len_no_ws) = "the noble and the singer"
    ///     .split_whitespace()
    ///     .better_collect_with_puller(
    ///         ConcatStr::new(),
    ///         |driver| driver.count(),
    ///     );
    ///
    /// assert_eq!(s_no_ws, "thenobleandthesinger");
    /// assert_eq!(len_no_ws, 5);
    /// ```
    #[cfg(feature = "unstable")]
    fn better_collect_with_puller<C, R>(
        self,
        mut collector: C,
        puller: impl FnOnce(Driver<'_, Self, C>) -> R,
    ) -> (C::Output, R)
    where
        Self: Sized,
        C: RefCollector<Item = Self::Item>,
    {
        let driver = assert_iterator(Driver::new(self, &mut collector));
        let ret = puller(driver);
        (collector.finish(), ret)
    }
}

impl<I: Iterator> BetterCollect for I {}
