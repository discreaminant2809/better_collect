//! [`Collector`]s for [`Vec`].
//!
//! This module corresponds to [`mod@std::vec`].
//!
//! [`Collector`]: crate::collector::Collector

use crate::{
    collector::{Collector, RefCollector},
    slice::{Concat, ConcatItem, ConcatItemSealed, ConcatSealed},
};

use std::{borrow::Borrow, ops::ControlFlow};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

/// A [`Collector`] that pushes collected items into a [`Vec`].
/// Its [`Output`] is [`Vec`].
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `Vec::into_collector()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
#[derive(Debug, Clone)]
pub struct IntoCollector<T>(Vec<T>);

/// A [`Collector`] that pushes collected items into a [`&mut Vec`](Vec).
/// Its [`Output`] is [`&mut Vec`](Vec).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `Vec::collector_mut()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
#[derive(Debug)]
pub struct CollectorMut<'a, T>(&'a mut Vec<T>);

impl<T> crate::collector::IntoCollector for Vec<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = IntoCollector<T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        IntoCollector(self)
    }
}

impl<'a, T> crate::collector::IntoCollector for &'a mut Vec<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = CollectorMut<'a, T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        CollectorMut(self)
    }
}

impl<T> Collector for IntoCollector<T> {
    type Item = T;
    type Output = Vec<T>;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.0.push(item);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.0.extend(items);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.0.extend(items);
        self.0
    }
}

impl<T: Copy> RefCollector for IntoCollector<T> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        self.0.push(*item);
        ControlFlow::Continue(())
    }
}

impl<'a, T> Collector for CollectorMut<'a, T> {
    type Item = T;
    type Output = &'a mut Vec<T>;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.0.push(item);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.0.extend(items);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.0.extend(items);
        self.0
    }
}

impl<T: Copy> RefCollector for CollectorMut<'_, T> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        self.0.push(*item);
        ControlFlow::Continue(())
    }
}

impl<T> Default for IntoCollector<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// # Examples
///
/// ```
/// use better_collect::prelude::*;
///
/// let matrix = [vec![1, 2], vec![3, 4, 5], vec![6]];
///
/// let array = matrix
///     .into_iter()
///     .feed_into(Vec::new().into_concat());
///
/// assert_eq!(array, [1, 2, 3, 4, 5, 6]);
/// ```
impl<T> Concat for Vec<T> {}

/// See [`std::slice::Concat`] for why this trait bound is used.
impl<S, T> ConcatItem<Vec<T>> for S
where
    S: Borrow<[T]>,
    T: Clone,
{
}

impl<T> ConcatSealed for Vec<T> {}

impl<S, T> ConcatItemSealed<Vec<T>> for S
where
    S: Borrow<[T]>,
    T: Clone,
{
    #[inline]
    fn push_to(&mut self, owned_slice: &mut Vec<T>) {
        owned_slice.extend_from_slice((*self).borrow());
    }
}
