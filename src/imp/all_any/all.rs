use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, assert_collector};

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
/// Since it does **not** implement [`RefCollector`], this collector should be used
/// as the **final collector** in a [`then`] chain, or adapted into a [`RefCollector`]
/// using the appropriate adaptor.
/// If you find yourself writing `All::new(...).cloned()` or `All::new(...).copied()`,
/// consider using [`AllRef`](crate::AllRef) instead, which avoids unnecessary cloning.
///
/// # Examples
///
/// ```
/// use better_collect::{Collector, All};
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
/// use better_collect::{Collector, All};
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
/// [`RefCollector`]: crate::RefCollector
/// [`then`]: crate::RefCollector::then
pub struct All<T, F> {
    inner: RawAllAny<F, true>,
    _marker: PhantomData<fn(T)>,
}

impl<T, F> All<T, F>
where
    F: FnMut(T) -> bool,
{
    /// Creates a new instance of this collector with the default output of `true`.
    #[inline]
    pub const fn new(pred: F) -> Self {
        assert_collector(Self {
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
