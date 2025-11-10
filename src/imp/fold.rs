use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, assert_collector};

/// A [`Collector`] that accumulates items using a function.
///
/// This collector corresponds to [`Iterator::fold()`], except that
/// the accumulated value is mutated in place.
///
/// Since it does **not** implement [`RefCollector`], this collector should be used
/// as the **final collector** in a [`then`] chain, or adapted into a [`RefCollector`]
/// using the appropriate adaptor.
/// If you find yourself writing `Fold::new(...).cloned()` or `Fold::new(...).copied()`,
/// consider using [`FoldRef`](crate::FoldRef) instead, which avoids unnecessary cloning.
///
/// # Examples
///
/// ```
/// use better_collect::{Collector, Fold};
///
/// let mut collector = Fold::new(0, |sum, num| *sum += num);
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert_eq!(collector.finish(), 6);
/// ```
///
/// [`RefCollector`]: crate::RefCollector
/// [`then`]: crate::RefCollector::then
pub struct Fold<A, T, F> {
    accum: A,
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<A, T, F> Fold<A, T, F>
where
    F: FnMut(&mut A, T),
{
    /// Creates a new instance of this collector with an initial value and an accumulator.
    pub const fn new(init: A, f: F) -> Self {
        assert_collector(Self {
            accum: init,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, T, F> Collector for Fold<A, T, F>
where
    F: FnMut(&mut A, T),
{
    type Item = T;

    type Output = A;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        items
            .into_iter()
            .for_each(|item| (self.f)(&mut self.accum, item));
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        items.into_iter().for_each({
            let accum = &mut self.accum;
            move |item| (self.f)(accum, item)
        });

        self.accum
    }
}

impl<A: Clone, T, F: Clone> Clone for Fold<A, T, F> {
    fn clone(&self) -> Self {
        Self {
            accum: self.accum.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.accum.clone_from(&source.accum);
        self.f.clone_from(&source.f);
    }
}

impl<A: Debug, T, F> Debug for Fold<A, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fold").field("accum", &self.accum).finish()
    }
}
