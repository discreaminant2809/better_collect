use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_ref_collector};

/// A [`RefCollector`] that accumulates items using a function.
///
/// This collector corresponds to [`Iterator::fold()`], except that
/// the accumulated value is mutated in place.
///
/// Unlike [`Fold`](crate::Fold), this adaptor only receives a mutable reference to each item.
/// Because of that, it can be used **in the middle** of a [`then`] chain,
/// since it is a [`RefCollector`].
/// While it can also appear at the end of the chain, consider using [`Fold`](crate::Fold)
/// there instead for better clarity.
///
/// # Examples
///
/// ```
/// use better_collect::{BetterCollect, RefCollector, FoldRef};
///
/// let (sum, _) = [1, 2, 3]
///     .into_iter()
///     .better_collect(
///         FoldRef::new(0, |sum, num| *sum += *num)
///             .then(vec![])
///     );
///
/// assert_eq!(sum, 6);
/// ```
///
/// [`then`]: crate::RefCollector::then
pub struct FoldRef<A, T, F> {
    accum: A,
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<A, T, F> FoldRef<A, T, F>
where
    F: FnMut(&mut A, &mut T),
{
    /// Creates a new instance of this collector with an initial value and an accumulator.
    pub const fn new(init: A, f: F) -> Self {
        assert_ref_collector(Self {
            accum: init,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, T, F> Collector for FoldRef<A, T, F>
where
    F: FnMut(&mut A, &mut T),
{
    type Item = T;

    type Output = A;

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        items
            .into_iter()
            .for_each(|mut item| (self.f)(&mut self.accum, &mut item));
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        items.into_iter().for_each({
            let accum = &mut self.accum;
            move |mut item| (self.f)(accum, &mut item)
        });

        self.accum
    }
}

impl<A, T, F> RefCollector for FoldRef<A, T, F>
where
    F: FnMut(&mut A, &mut T),
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item);
        ControlFlow::Continue(())
    }
}

impl<A: Clone, T, F: Clone> Clone for FoldRef<A, T, F> {
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

impl<A: Debug, T, F> Debug for FoldRef<A, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FoldRef")
            .field("accum", &self.accum)
            .finish()
    }
}
