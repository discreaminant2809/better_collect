//! [`Collector`]s for [`VecDeque`]
//!
//! This module corresponds to [`std::collections::vec_deque`].
//!
//! [`Collector`]: crate::Collector

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(feature = "std")]
use std::collections::VecDeque;

/// A [`Collector`] that pushes collected items into the back of a [`VecDeque`].
/// Its [`Output`] is [`VecDeque`].
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `VecDeque::into_collector()`.
///
/// [`Collector`]: crate::Collector
/// [`Output`]: crate::Collector::Output
/// [`RefCollector`]: crate::RefCollector
pub struct IntoCollector<T>(pub(super) VecDeque<T>);

/// A [`Collector`] that pushes collected items into the back of a [`&mut VecDeque`](VecDeque).
/// Its [`Output`] is [`&mut VecDeque`](VecDeque).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `VecDeque::collector_mut()`.
///
/// [`Collector`]: crate::Collector
/// [`Output`]: crate::Collector::Output
/// [`RefCollector`]: crate::RefCollector
pub struct CollectorMut<'a, T>(pub(super) &'a mut VecDeque<T>);
