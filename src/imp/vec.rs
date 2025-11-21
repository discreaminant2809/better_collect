//! A [`Collector`] for [`Vec`].
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
pub struct IntoCollector<T>(Vec<T>);

/// A [`Collector`] that pushes collected items into a [`Vec`].
/// Its [`Output`] is [`&mut Vec`](Vec).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by [`Vec::collector()`](VecExt::collector).
///
/// [`Collector`]: crate::Collector
/// [`Output`]: crate::Collector::Output
pub struct Collector<'a, T>(&'a mut Vec<T>);

/// Extends methods for [`Vec`].
#[allow(private_bounds)]
pub trait VecExt<T>: Sealed {
    /// Creates a [`Collector`] that pushes collected items into a [`Vec`].
    /// The `Vec` is mutably borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{Collector, vec::VecExt};
    ///
    /// let mut v = vec![1, 1];
    /// let mut collector = v.collector();
    /// collector.collect_many([1; 3]);
    ///
    /// assert_eq!(v, [1; 5]);
    /// ```
    ///
    /// [`Collector`]: crate::Collector
    fn collector(&mut self) -> Collector<'_, T>;
}

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

    type IntoCollector = Collector<'a, T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        Collector(self)
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

impl<'a, T> crate::Collector for Collector<'a, T> {
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

impl<T: Copy> RefCollector for Collector<'_, T> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        self.0.push(*item);
        ControlFlow::Continue(())
    }
}

impl<T> VecExt<T> for Vec<T> {
    #[inline]
    fn collector(&mut self) -> Collector<'_, T> {
        Collector(self)
    }
}

trait Sealed {}

impl<T> Sealed for Vec<T> {}
