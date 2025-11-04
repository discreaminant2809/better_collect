use std::ops::ControlFlow;

use crate::{Collector, assert_collector};

/// A [`Collector`] that computes the minimum value among the items it collects.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the minimum item otherwise.
///
/// This collector corresponds to [`Iterator::min()`].
///
/// # Examples
///
/// ```
/// use better_collect::{Collector, cmp::Min};
///
/// let mut collector = Min::new();
///
/// assert!(collector.collect(5).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert_eq!(collector.finish(), Some(1));
/// ```
///
/// Its output is `None` if it has not encountered any items.
///
/// ```
/// use better_collect::{Collector, cmp::Min};
///
/// assert_eq!(Min::<i32>::new().finish(), None);
/// ```
#[derive(Debug, Clone)]
pub struct Min<T> {
    // For `Debug` impl for `MinByKey`.
    pub(super) min: Option<T>,
}

impl<T: Ord> Min<T> {
    /// Creates a new instance of this collector.
    #[inline]
    pub const fn new() -> Self {
        assert_collector(Self { min: None })
    }
}

impl<T: Ord> Default for Min<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Collector for Min<T> {
    type Item = T;
    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.min = Some(match self.min.take() {
            Some(min) => min.min(item),
            None => item,
        });

        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.min
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.min = self.min.take().into_iter().chain(items).min();
        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.min.into_iter().chain(items).min()
    }
}
