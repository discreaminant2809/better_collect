use std::ops::ControlFlow;

use crate::{Collector, assert_collector};

/// A [`Collector`] that computes the maximum value among the items it collects.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the maximum item otherwise.
///
/// This collector corresponds to [`Iterator::max()`].
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, cmp::Max};
///
/// let mut collector = Max::new();
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(3).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(5).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert_eq!(collector.finish(), Some(5));
/// ```
///
/// The output is `None` if no items were collected.
///
/// ```
/// use better_collect::{prelude::*, cmp::Max};
///
/// assert_eq!(Max::<i32>::new().finish(), None);
/// ```
#[derive(Debug, Clone)]
pub struct Max<T> {
    // For `Debug` impl used by `MaxByKey`.
    pub(super) max: Option<T>,
}

impl<T: Ord> Max<T> {
    /// Creates a new instance of this collector.
    #[inline]
    pub const fn new() -> Self {
        assert_collector(Self { max: None })
    }
}

impl<T: Ord> Default for Max<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Collector for Max<T> {
    type Item = T;
    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        // Because it's `Max`, if `max` is a `None` then it's always smaller than a `Some`.
        // Doesn't work on `Min`, however.
        // Be careful to preserve the semantics of `Iterator::max` that if there are
        // more than one maximum values, the last one is chosen.
        self.max = self.max.take().max(Some(item));
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.max
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.max = self.max.take().into_iter().chain(items).max();
        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.max.into_iter().chain(items).max()
    }
}
