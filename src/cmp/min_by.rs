use std::{
    cmp::{Ordering, min_by},
    fmt::Debug,
    ops::ControlFlow,
};

use crate::collector::{Collector, CollectorBase, assert_collector};

/// A [`Collector`] that computes the minimum value among the items it collects
/// according to a comparison function.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the minimum item otherwise.
///
/// This collector is constructed by [`Min::by()`](super::Min::by).
///
/// This collector corresponds to [`Iterator::min_by()`].
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, cmp::Min};
///
/// let mut collector = Min::by(f64::total_cmp);
///
/// assert!(collector.collect(1.1).is_continue());
/// assert!(collector.collect(-2.3).is_continue());
/// assert!(collector.collect(f64::INFINITY).is_continue());
/// assert!(collector.collect(-1E2).is_continue());
/// assert!(collector.collect((-1_f64).sin()).is_continue());
///
/// assert_eq!(collector.finish(), Some(-1E2));
/// ```
///
/// The output is `None` if no items were collected.
///
/// ```
/// use better_collect::{prelude::*, cmp::Min};
///
/// assert_eq!(Min::by(f64::total_cmp).finish(), None);
/// ```
#[derive(Clone)]
pub struct MinBy<T, F> {
    min: Option<T>,
    f: F,
}

impl<T, F> MinBy<T, F>
where
    F: FnMut(&T, &T) -> Ordering,
{
    /// Creates a new instance of this collector with a given comparison function.
    #[deprecated(since = "0.3.0", note = "Use `Min::by`")]
    #[inline]
    pub const fn new(f: F) -> Self {
        assert_collector(Self { min: None, f })
    }
}

impl<T, F> CollectorBase for MinBy<T, F> {
    type Output = Option<T>;

    #[inline]
    fn finish(self) -> Self::Output {
        self.min
    }
}

impl<T, F> Collector<T> for MinBy<T, F>
where
    F: FnMut(&T, &T) -> Ordering,
{
    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.min = Some(match self.min.take() {
            Some(min) => min_by(min, item, &mut self.f),
            None => item,
        });

        ControlFlow::Continue(())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.min = self.min.take().into_iter().chain(items).min_by(&mut self.f);
        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.min.into_iter().chain(items).min_by(self.f)
    }
}

impl<T: Debug, F> Debug for MinBy<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinBy").field("min", &self.min).finish()
    }
}
