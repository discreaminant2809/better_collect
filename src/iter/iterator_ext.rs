#[cfg(feature = "unstable")]
use super::Driver;

#[cfg(feature = "unstable")]
use crate::assert_iterator;
#[cfg(feature = "unstable")]
use crate::collector::RefCollector;
use crate::collector::{Collector, IntoCollector};

/// Extends [`Iterator`] with the [`better_collect`](IteratorExt::better_collect) method
/// for working seamlessly with [`Collector`]s.
///
/// This trait is automatically implemented for all [`Iterator`] types.
pub trait IteratorExt: Iterator {
    /// Extracts items from this iterator into the provided collector till
    /// the collector stops accumulating or the iterator is exhausted.
    /// and returns the collector’s output.
    ///
    /// To use this method, import the [`IteratorExt`] trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{prelude::*, cmp::Max};
    ///
    /// let (nums, max) = [4, 2, 6, 3]
    ///     .into_iter()
    ///     .better_collect(vec![].into_collector().combine(Max::new()));
    ///
    /// assert_eq!(nums, [4, 2, 6, 3]);
    /// assert_eq!(max, Some(6));
    /// ```
    #[inline]
    fn better_collect<C>(&mut self, collector: C) -> C::Output
    where
        C: IntoCollector<Item = Self::Item>,
    {
        collector.into_collector().collect_then_finish(self)
    }

    /// Extracts items from this iterator into the provided collector as far as the
    /// puller drives the iterator, then returns both the collector’s output and
    /// the puller’s result.
    ///
    /// The `puller` is a closure that receives an [`Iterator`] as a *driver*
    /// and produces an additional result.
    /// An item is collected only when the driver advances pass that item.
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
    /// To use this method, import the [`IteratorExt`] trait.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use better_collect::prelude::*;
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
        collector: C,
        puller: impl FnOnce(Driver<'_, Self, C::IntoCollector>) -> R,
    ) -> (C::Output, R)
    where
        Self: Sized,
        C: IntoCollector<Item = Self::Item, IntoCollector: RefCollector>,
    {
        let mut collector = collector.into_collector();
        let driver = assert_iterator(Driver::new(self, &mut collector));
        let ret = puller(driver);
        (collector.finish(), ret)
    }
}

impl<I: Iterator> IteratorExt for I {}
