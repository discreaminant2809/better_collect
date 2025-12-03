//! [`Collector`]s for [`BinaryHeap`]
//!
//! This module corresponds to [`std::collections::binary_heap`].
//!
//! # Notes
//!
//! You generally should **not** use `BinaryHeap::new().into_collector()`
//! to construct a new `BinaryHeap`, since the time complexity is `O(n log n)`,
//! which can be less efficient than constructing a [`Vec`] then converting to
//! a `BinaryHeap` (`O(n)`).
//!
//! [`Collector`]: crate::Collector

#[cfg(not(feature = "std"))]
use alloc::collections::BinaryHeap;
#[cfg(feature = "std")]
use std::collections::BinaryHeap;

/// A [`Collector`] that pushes collected items into a [`BinaryHeap`].
/// Its [`Output`] is [`BinaryHeap`].
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `BinaryHeap::into_collector()`.
///
/// [`Collector`]: crate::Collector
/// [`Output`]: crate::Collector::Output
/// [`RefCollector`]: crate::RefCollector
pub struct IntoCollector<T>(pub(super) BinaryHeap<T>);

/// A [`Collector`] that pushes collected items into a [`&mut BinaryHeap`](BinaryHeap).
/// Its [`Output`] is [`&mut BinaryHeap`](BinaryHeap).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `BinaryHeap::collector_mut()`.
///
/// [`Collector`]: crate::Collector
/// [`Output`]: crate::Collector::Output
/// [`RefCollector`]: crate::RefCollector
pub struct CollectorMut<'a, T>(pub(super) &'a mut BinaryHeap<T>);
