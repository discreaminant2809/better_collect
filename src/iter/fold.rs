use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{
    assert_collector, assert_ref_collector,
    collector::{Collector, RefCollector},
};

/// A [`Collector`] that accumulates items using a function.
///
/// This collector corresponds to [`Iterator::fold()`], except that
/// the accumulated value is mutated in place.
///
/// This collector has a `Ref` counterpart created by [`new_ref()`](Fold::new_ref).
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, Fold};
///
/// let mut collector = Fold::new(0, |sum, num| *sum += num);
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert_eq!(collector.finish(), 6);
/// ```
pub struct Fold<A, T, F> {
    accum: A,
    f: F,
    _marker: PhantomData<fn(T)>,
}

/// A [`RefCollector`] that accumulates items using a function.
///
/// This is the `Ref` counterpart and shares the same semantics as [`Fold`].
/// Ses its documentation for more.
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, Fold};
///
/// let (sum, _) = [1, 2, 3]
///     .into_iter()
///     .better_collect(
///         Fold::new_ref(0, |sum, num| *sum += *num)
///             .combine(vec![])
///     );
///
/// assert_eq!(sum, 6);
/// ```
pub struct FoldRef<A, T, F> {
    accum: A,
    f: F,
    _marker: PhantomData<fn(&mut T)>,
}

impl<A, T, F> Fold<A, T, F> {
    /// Creates a new instance of this collector with an initial value and an accumulator.
    pub const fn new(init: A, f: F) -> Self
    where
        F: FnMut(&mut A, T),
    {
        assert_collector(Self {
            accum: init,
            f,
            _marker: PhantomData,
        })
    }

    /// Creates a new instance of the `Ref` counterpart of this collector
    /// with an initial value and an accumulator.
    pub const fn new_ref(init: A, f: F) -> FoldRef<A, T, F>
    where
        F: FnMut(&mut A, &mut T),
    {
        assert_ref_collector(FoldRef {
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
