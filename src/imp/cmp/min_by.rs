use std::{
    cmp::{Ordering, min_by},
    fmt::Debug,
    ops::ControlFlow,
};

use crate::{Collector, assert_collector};

/// A [`Collector`] that computes the minimum value among the items it collects
/// according to a comparison function.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the minimum item otherwise.
///
/// This collector corresponds to [`Iterator::min_by()`].
///
/// # Examples
///
/// ```
/// use better_collect::{Collector, cmp::MinBy};
///
/// let mut collector = MinBy::new(f64::total_cmp);
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
/// use better_collect::{Collector, cmp::MinBy};
///
/// assert_eq!(MinBy::new(f64::total_cmp).finish(), None);
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
    #[inline]
    pub const fn new(f: F) -> Self {
        assert_collector(Self { min: None, f })
    }
}

impl<T, F> Collector for MinBy<T, F>
where
    F: FnMut(&T, &T) -> Ordering,
{
    type Item = T;

    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.min = Some(match self.min.take() {
            Some(min) => min_by(min, item, &mut self.f),
            None => item,
        });

        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.min
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        if let Some(min) = items.into_iter().min_by(&mut self.f) {
            self.collect(min)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.min.into_iter().chain(items).min_by(self.f)
    }
}

impl<T: Debug, F> Debug for MinBy<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinBy").field("min", &self.min).finish()
    }
}
