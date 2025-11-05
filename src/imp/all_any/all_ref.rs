use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_ref_collector};

use super::raw_all_any::RawAllAny;

/// A [`RefCollector`] that tests whether all collected items satisfy a predicate.
///
/// Its [`Output`] is initially `true` and remains `true` as long as every collected item
/// satisfies the predicate.
/// When the collector collects an item that makes the predicate `false`,
/// it returns [`Break`], and the [`Output`] becomes `false`.
///
/// This collector corresponds to [`Iterator::all()`].
///
/// Unlike [`All`], this adaptor only receives a mutable reference to each item.
/// Because of that, it can be used **in the middle** of a [`then`] chain,
/// since it is a [`RefCollector`].
/// While it can also appear at the end of the chain, consider using [`All`]
/// there instead for better clarity.
///
/// # Examples
///
/// ```
/// use better_collect::{
///     BetterCollect, RefCollector,
///     AllRef, num::Sum
/// };
///
/// let (all_even, sum) = [2, 4, 6]
///     .into_iter()
///     .better_collect(
///         AllRef::new(|&mut x| x % 2 == 0)
///             .then(Sum::<i32>::new())
///     );
///
/// assert!(all_even);
/// assert_eq!(sum, 12);
/// ```
///
/// ```
/// use better_collect::{
///     BetterCollect, RefCollector,
///     AllRef, num::Sum
/// };
///
/// let (all_even, sum) = [2, 5, 6]
///     .into_iter()
///     .better_collect(
///         AllRef::new(|&mut x| x % 2 == 0)
///             .then(Sum::<i32>::new())
///     );
///
/// assert!(!all_even);
/// assert_eq!(sum, 13);
/// ```
///
/// [`Break`]: std::ops::ControlFlow::Break
/// [`Output`]: Collector::Output
/// [`RefCollector`]: crate::RefCollector
/// [`then`]: crate::RefCollector::then
/// [`All`]: crate::All
pub struct AllRef<T, F> {
    inner: RawAllAny<F, true>,
    _marker: PhantomData<fn(&mut T)>,
}

impl<T, F> AllRef<T, F>
where
    F: FnMut(&mut T) -> bool,
{
    /// Creates a new instance of this collector with the default output of `true`.
    #[inline]
    pub const fn new(pred: F) -> Self {
        assert_ref_collector(Self {
            inner: RawAllAny::new(pred),
            _marker: PhantomData,
        })
    }

    /// Returns the current result of the accumulation.
    #[inline]
    pub const fn get(&self) -> bool {
        self.inner.get()
    }
}

impl<T, F> Collector for AllRef<T, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

    type Output = bool;

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        self.inner.collect_impl(move |pred| pred(&mut item))
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.get()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.inner
            .collect_impl(|pred| items.into_iter().all(|mut item| pred(&mut item)))
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.inner.collect_then_finish_impl(|mut pred| {
            items.into_iter().all(move |mut item| pred(&mut item))
        })
    }
}

impl<T, F> RefCollector for AllRef<T, F>
where
    F: FnMut(&mut T) -> bool,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.inner.collect_impl(move |pred| pred(item))
    }
}

impl<T, F: Clone> Clone for AllRef<T, F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
    }
}

impl<T, F> Debug for AllRef<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.debug_impl(f.debug_struct("AllRef"))
    }
}
