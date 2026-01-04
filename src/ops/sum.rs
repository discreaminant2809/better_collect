use std::{fmt::Debug, iter, marker::PhantomData, ops::AddAssign, ops::ControlFlow};

use crate::collector::Collector;

/// A [`Collector`] that computes the sum of all collected items.
///
/// It is generic over two types:
///
/// - `A`: the accumulator and final result type.
///   Must implement both [`Sum<T>`] and [`AddAssign<Self>`].
/// - `T`: the item type that this collector consumes.
///
/// This collector corresponds to [`Iterator::sum()`], except that its return type
/// is slightly more restrictive (additionally requiring [`AddAssign<Self>`]).
///
/// Because this is an “umbrella” implementation which has more generics than needed
/// (cumbersome to initialize) and does **not** implement [`RefCollector`],
/// you can define your own `Sum` collector tailored to your type, with fewer generics
/// and optional [`RefCollector`] implementation.
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, Sum};
///
/// let mut collector = Sum::<i32, _>::new();
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert_eq!(collector.finish(), 6);
/// ```
///
/// [`RefCollector`]: crate::collector::RefCollector
pub struct Sum<A, T> {
    sum: A,
    _marker: PhantomData<fn(T)>,
}

impl<A: iter::Sum<T> + AddAssign, T> Sum<A, T> {
    /// Create a new instance of this collector with the initial value being
    /// the *additive identity* (“zero”) of the type.
    #[inline]
    pub fn new() -> Self {
        Self {
            sum: None.into_iter().sum(),
            _marker: PhantomData,
        }
    }
}

impl<A: iter::Sum<T> + AddAssign, T> Collector for Sum<A, T> {
    type Item = T;

    type Output = A;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.collect_many(Some(item))
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.sum
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.sum += items.into_iter().sum::<A>();
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.sum += items.into_iter().sum::<A>();
        self.sum
    }
}

impl<A: iter::Sum<T> + AddAssign, T> Default for Sum<A, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Clone, T> Clone for Sum<A, T> {
    fn clone(&self) -> Self {
        Self {
            sum: self.sum.clone(),
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.sum.clone_from(&source.sum);
    }
}

impl<A: Debug, T> Debug for Sum<A, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sum").field("sum", &self.sum).finish()
    }
}
