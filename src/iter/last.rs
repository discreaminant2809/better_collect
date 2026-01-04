use std::ops::ControlFlow;

use crate::{assert_collector, collector::Collector};

/// A [`Collector`] that stores the last item it collects.
///
/// If no items have been collected, its [`Output`] is `None`;
/// otherwise, it is `Some` containing the most recently collected item.
///
/// This collector corresponds to [`Iterator::last()`].
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, Last};
///
/// let mut collector = Last::new();
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert_eq!(collector.finish(), Some(3));
/// ```
///
/// ```
/// use better_collect::{prelude::*, Last};
///
/// assert_eq!(Last::<i32>::new().finish(), None);
/// ```
///
/// [`Output`]: Collector::Output
#[derive(Debug, Clone)]
pub struct Last<T> {
    value: Option<T>,
}

impl<T> Last<T> {
    /// Creates an intance of this collector.
    #[inline]
    pub const fn new() -> Self {
        assert_collector(Last { value: None })
    }
}

impl<T> Collector for Last<T> {
    type Item = T;
    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.value = Some(item);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.value
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // We need a bit complication here since we may risk assigning `None` to `self.value` being `Some`.
        match (&mut self.value, items.into_iter().last()) {
            (Some(value), Some(last)) => *value = last,
            // DO NOT update here. `items` don't have a value to "inherit" the last spot.
            (Some(_), None) => {}
            (None, last) => self.value = last,
        }

        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        items.into_iter().last().or(self.value)
    }
}

impl<T> Default for Last<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
