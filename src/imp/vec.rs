//! [`Collector`]s for [`Vec`].
//!
//! This module corresponds to [`mod@std::vec`].
//!
//! [`Collector`]: crate::Collector

use crate::RefCollector;

use std::ops::ControlFlow;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

/// A [`Collector`] that pushes collected items into a [`Vec`].
/// Its [`Output`] is [`Vec`].
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `Vec::into_collector()`.
///
/// [`Collector`]: crate::Collector
/// [`Output`]: crate::Collector::Output
#[derive(Debug, Default, Clone)]
pub struct IntoCollector<T>(Vec<T>);

/// A [`Collector`] that pushes collected items into a [`&mut Vec`](Vec).
/// Its [`Output`] is [`&mut Vec`](Vec).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `Vec::collector_mut()`.
///
/// [`Collector`]: crate::Collector
/// [`Output`]: crate::Collector::Output
pub struct CollectorMut<'a, T>(&'a mut Vec<T>);

impl<T> crate::IntoCollector for Vec<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = IntoCollector<T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        IntoCollector(self)
    }
}

impl<'a, T> crate::IntoCollector for &'a mut Vec<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = CollectorMut<'a, T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        CollectorMut(self)
    }
}

impl<T> crate::Collector for IntoCollector<T> {
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

impl<'a, T> crate::Collector for CollectorMut<'a, T> {
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
