use crate::{assert_collector, collector::Collector};

use std::{fmt::Debug, ops::ControlFlow};

/// A [`Collector`] that reduces all collected items into a single value
/// by repeatedly applying a reduction function.
///
/// If no items have been collected, its [`Output`](crate::collector::Collector::Output) is `None`;
/// otherwise, it returns `Some` containing the result of the reduction.
///
/// This collector corresponds to [`Iterator::reduce()`].
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, iter::Reduce};
///
/// let mut collector = Reduce::new(|accum, num| accum + num);
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(3).is_continue());
/// assert!(collector.collect(5).is_continue());
///
/// assert_eq!(collector.finish(), Some(9));
/// ```
///
/// The output is `None` if no items were collected.
///
/// ```
/// use better_collect::{prelude::*, iter::Reduce};
///
/// assert_eq!(Reduce::new(|accum: i32, num| accum + num).finish(), None);
/// ```
#[derive(Clone)]
pub struct Reduce<T, F> {
    accum: Option<T>,
    f: F,
}

impl<T, F> Reduce<T, F>
where
    F: FnMut(T, T) -> T,
{
    /// Crates a new instance of this collector with a given accumulator.
    #[inline]
    pub const fn new(f: F) -> Self {
        assert_collector(Self { accum: None, f })
    }
}

impl<T, F> Collector for Reduce<T, F>
where
    F: FnMut(T, T) -> T,
{
    type Item = T;

    type Output = Option<T>;

    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        if let Some(accum) = self.accum.take() {
            self.accum = Some((self.f)(accum, item));
        } else {
            self.accum = Some(item);
        };

        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.accum = self
            .accum
            .take()
            .into_iter()
            .chain(items)
            .reduce(&mut self.f);

        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.accum.into_iter().chain(items).reduce(self.f)
    }
}

impl<T: Debug, F> Debug for Reduce<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reduce")
            .field("accum", &self.accum)
            .finish()
    }
}
