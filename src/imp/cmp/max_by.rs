use std::{
    cmp::{Ordering, max_by},
    fmt::Debug,
    ops::ControlFlow,
};

use crate::{Collector, assert_collector};

/// A [`Collector`] that computes the maximum value among the items it collects
/// according to a comparison function.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the maximum item otherwise.
///
/// This collector corresponds to [`Iterator::max_by()`].
///
/// # Examples
///
/// ```
/// use better_collect::{Collector, cmp::MaxBy};
///
/// let mut collector = MaxBy::new(f64::total_cmp);
///
/// assert!(collector.collect(1.1).is_continue());
/// assert!(collector.collect(-2.3).is_continue());
/// assert!(collector.collect(f64::NEG_INFINITY).is_continue());
/// assert!(collector.collect(1E2).is_continue());
/// assert!(collector.collect(99.0_f64.sqrt()).is_continue());
///
/// assert_eq!(collector.finish(), Some(1E2));
/// ```
///
/// The output is `None` if no items were collected.
///
/// ```
/// use better_collect::{Collector, cmp::MaxBy};
///
/// assert_eq!(MaxBy::new(f64::total_cmp).finish(), None);
/// ```
#[derive(Clone)]
pub struct MaxBy<T, F> {
    max: Option<T>,
    f: F,
}

impl<T, F> MaxBy<T, F>
where
    F: FnMut(&T, &T) -> Ordering,
{
    /// Creates a new instance of this collector with a given comparison function.
    #[inline]
    pub const fn new(f: F) -> Self {
        assert_collector(Self { max: None, f })
    }
}

impl<T, F> Collector for MaxBy<T, F>
where
    F: FnMut(&T, &T) -> Ordering,
{
    type Item = T;

    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.max = Some(match self.max.take() {
            Some(max) => max_by(max, item, &mut self.f),
            None => item,
        });

        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.max
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.max = self.max.take().into_iter().chain(items).max_by(&mut self.f);
        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.max.into_iter().chain(items).max_by(self.f)
    }
}

impl<T: Debug, F> Debug for MaxBy<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaxBy").field("max", &self.max).finish()
    }
}
