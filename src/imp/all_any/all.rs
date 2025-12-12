use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_collector, assert_ref_collector};

use super::raw_all_any::RawAllAny;

/// A [`Collector`] that tests whether all collected items satisfy a predicate.
///
/// Its [`Output`] is initially `true` and remains `true` as long as every collected item
/// satisfies the predicate.
/// When the collector collects an item that makes the predicate `false`,
/// it returns [`Break`], and the [`Output`] becomes `false`.
///
/// This collector corresponds to [`Iterator::all()`].
///
/// This collector has a `Ref` counterpart created by [`new_ref()`](All::new_ref).
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, All};
///
/// let mut collector = All::new(|x| x > 0);
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert!(collector.finish());
/// ```
///
/// ```
/// use better_collect::{prelude::*, All};
///
/// let mut collector = All::new(|x| x > 0);
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
///
/// // First mismatched item.
/// assert!(collector.collect(-1).is_break());
///
/// assert!(!collector.finish());
/// ```
///
/// [`Break`]: std::ops::ControlFlow::Break
/// [`Output`]: Collector::Output
pub struct All<T, F> {
    inner: RawAllAny<F, true>,
    _marker: PhantomData<fn(T)>,
}

/// A [`RefCollector`] that tests whether all collected items satisfy a predicate.
///
/// This is the `Ref` counterpart and shares the same semantics as [`All`].
/// Ses its documentation for more.
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, All, num::Sum};
///
/// let (all_even, sum) = [2, 4, 6]
///     .into_iter()
///     .better_collect(
///         All::new_ref(|&mut x| x % 2 == 0)
///             .combine(Sum::<i32>::new())
///     );
///
/// assert!(all_even);
/// assert_eq!(sum, 12);
/// ```
///
/// ```
/// use better_collect::{prelude::*, All, num::Sum};
///
/// let (all_even, sum) = [2, 5, 6]
///     .into_iter()
///     .better_collect(
///         All::new_ref(|&mut x| x % 2 == 0)
///             .combine(Sum::<i32>::new())
///     );
///
/// assert!(!all_even);
/// assert_eq!(sum, 13);
/// ```
pub struct AllRef<T, F> {
    inner: RawAllAny<F, true>,
    _marker: PhantomData<fn(&mut T)>,
}

impl<T, F> All<T, F> {
    /// Creates a new instance of this collector with the default output of `true`.
    #[inline]
    pub const fn new(pred: F) -> Self
    where
        F: FnMut(T) -> bool,
    {
        assert_collector(Self {
            inner: RawAllAny::new(pred),
            _marker: PhantomData,
        })
    }

    /// Creates a new instance of the `Ref` counterpart of this collector
    /// with the default output of `true`.
    #[inline]
    pub const fn new_ref(pred: F) -> AllRef<T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        assert_ref_collector(AllRef {
            inner: RawAllAny::new(pred),
            _marker: PhantomData,
        })
    }
}

impl<T, F> All<T, F>
where
    F: FnMut(T) -> bool,
{
    /// Returns the current result of the accumulation.
    #[inline]
    pub const fn get(&self) -> bool {
        self.inner.get()
    }
}

impl<T, F> AllRef<T, F>
where
    F: FnMut(&mut T) -> bool,
{
    /// Returns the current result of the accumulation.
    #[inline]
    pub const fn get(&self) -> bool {
        self.inner.get()
    }
}

impl<T, F> Collector for All<T, F>
where
    F: FnMut(T) -> bool,
{
    type Item = T;

    type Output = bool;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.inner.collect_impl(|pred| pred(item))
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.get()
    }

    #[inline]
    fn break_hint(&self) -> bool {
        self.inner.has_stopped()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.inner.collect_impl(|pred| items.into_iter().all(pred))
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.inner
            .collect_then_finish_impl(|pred| items.into_iter().all(pred))
    }
}

impl<T, F: Clone> Clone for All<T, F> {
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

impl<T, F> Debug for All<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.debug_impl(f.debug_struct("All"))
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
    fn break_hint(&self) -> bool {
        self.inner.has_stopped()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.inner
            .collect_impl(|pred| items.into_iter().all(move |mut item| pred(&mut item)))
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
