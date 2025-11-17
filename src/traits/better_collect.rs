use crate::Collector;

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
    ///     BetterCollect, Collector, RefCollector,
    ///     cmp::Max,
    /// };
    ///
    /// let (nums, max) = [4, 2, 6, 3]
    ///     .into_iter()
    ///     .better_collect(vec![].then(Max::new()));
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
}

impl<I: Iterator> BetterCollect for I {}
