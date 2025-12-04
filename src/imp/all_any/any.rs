use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_collector, assert_ref_collector};

use super::raw_all_any::RawAllAny;

/// A [`Collector`] that tests whether any collected item satisfies a predicate.
///
/// Its [`Output`] is initially `false` and remains `false` as long as every collected item
/// does **not** satisfy the predicate.
/// When the collector collects an item that makes the predicate `true`,
/// it returns [`Break`], and the [`Output`] becomes `true`.
///
/// This collector corresponds to [`Iterator::any()`].
///
/// This collector has a `Ref` counterpart created by [`new_ref()`](Any::new_ref).
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, Any};
///
/// let mut collector = Any::new(|x| x < 0);
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert!(!collector.finish());
/// ```
///
/// ```
/// use better_collect::{prelude::*, Any};
///
/// let mut collector = Any::new(|x| x < 0);
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
///
/// // First matched item.
/// assert!(collector.collect(-1).is_break());
///
/// assert!(collector.finish());
/// ```
///
/// [`Break`]: std::ops::ControlFlow::Break
/// [`Output`]: Collector::Output
pub struct Any<T, F> {
    inner: RawAllAny<F, false>,
    _marker: PhantomData<fn(T)>,
}

/// A [`RefCollector`] that tests whether any collected item satisfies a predicate.
///
/// This is the `Ref` counterpart of [`Any`] and shares the same semantics.
/// Ses its documentation for more.
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, Any, num::Sum};
///
/// let (any_even, sum) = [1, 3, 5]
///     .into_iter()
///     .better_collect(
///         Any::new_ref(|&mut x| x % 2 == 0)
///             .then(Sum::<i32>::new())
///     );
///
/// assert!(!any_even);
/// assert_eq!(sum, 9);
/// ```
///
/// ```
/// use better_collect::{prelude::*, Any, num::Sum};
///
/// let (any_even, sum) = [1, 2, 5]
///     .into_iter()
///     .better_collect(
///         Any::new_ref(|&mut x| x % 2 == 0)
///             .then(Sum::<i32>::new())
///     );
///
/// assert!(any_even);
/// assert_eq!(sum, 8);
/// ```
pub struct AnyRef<T, F> {
    inner: RawAllAny<F, false>,
    _marker: PhantomData<fn(&mut T)>,
}

impl<T, F> Any<T, F> {
    /// Creates a new instance of this collector with the default output of `false`.
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
    /// with the default output of `false`.
    #[inline]
    pub const fn new_ref(pred: F) -> AnyRef<T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        assert_ref_collector(AnyRef {
            inner: RawAllAny::new(pred),
            _marker: PhantomData,
        })
    }
}

impl<T, F> Any<T, F>
where
    F: FnMut(T) -> bool,
{
    /// Returns the current result of the accumulation.
    #[inline]
    pub const fn get(&self) -> bool {
        self.inner.get()
    }
}

impl<T, F> AnyRef<T, F>
where
    F: FnMut(&mut T) -> bool,
{
    /// Returns the current result of the accumulation.
    #[inline]
    pub const fn get(&self) -> bool {
        self.inner.get()
    }
}

impl<T, F> Collector for Any<T, F>
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
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.inner.collect_impl(|pred| items.into_iter().any(pred))
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.inner
            .collect_then_finish_impl(|pred| items.into_iter().any(pred))
    }
}

impl<T, F: Clone> Clone for Any<T, F> {
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

impl<T, F> Debug for Any<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.debug_impl(f.debug_struct("Any"))
    }
}

impl<T, F> Collector for AnyRef<T, F>
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
            .collect_impl(|pred| items.into_iter().any(|mut item| pred(&mut item)))
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.inner.collect_then_finish_impl(|mut pred| {
            items.into_iter().any(move |mut item| pred(&mut item))
        })
    }
}

impl<T, F> RefCollector for AnyRef<T, F>
where
    F: FnMut(&mut T) -> bool,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.inner.collect_impl(move |pred| pred(item))
    }
}

impl<T, F: Clone> Clone for AnyRef<T, F> {
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

impl<T, F> Debug for AnyRef<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.debug_impl(f.debug_struct("AnyRef"))
    }
}
