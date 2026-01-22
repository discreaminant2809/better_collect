use std::{
    cmp::{Ordering, max_by},
    fmt::Debug,
    ops::ControlFlow,
};

use crate::collector::{Collector, CollectorBase, assert_collector};

/// A [`Collector`] that computes the maximum value among the items it collects
/// according to a comparison function.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the maximum item otherwise.
///
/// This collector is constructed by [`Max::by()`](super::Max::by).
///
/// This collector corresponds to [`Iterator::max_by()`].
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, cmp::Max};
///
/// let mut collector = Max::by(f64::total_cmp);
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
/// use better_collect::{prelude::*, cmp::Max};
///
/// assert_eq!(Max::by(f64::total_cmp).finish(), None);
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
    #[deprecated(since = "0.3.0", note = "Use `Max::by`")]
    #[inline]
    pub const fn new(f: F) -> Self {
        assert_collector(Self { max: None, f })
    }
}

impl<T, F> CollectorBase for MaxBy<T, F> {
    type Output = Option<T>;

    #[inline]
    fn finish(self) -> Self::Output {
        self.max
    }
}

impl<T, F> Collector<T> for MaxBy<T, F>
where
    F: FnMut(&T, &T) -> Ordering,
{
    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.max = Some(match self.max.take() {
            Some(max) => max_by(max, item, &mut self.f),
            None => item,
        });

        ControlFlow::Continue(())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.max = self.max.take().into_iter().chain(items).max_by(&mut self.f);
        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.max.into_iter().chain(items).max_by(self.f)
    }
}

impl<T: Debug, F> Debug for MaxBy<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaxBy").field("max", &self.max).finish()
    }
}
