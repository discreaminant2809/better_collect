//! [`Collector`]s for [`VecDeque`]
//!
//! This module corresponds to [`std::collections::vec_deque`].
//!
//! [`Collector`]: crate::collector::Collector

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
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
#[derive(Debug, Clone)]
pub struct IntoCollector<T>(pub(super) VecDeque<T>);

/// A [`Collector`] that pushes collected items into the back of a [`&mut VecDeque`](VecDeque).
/// Its [`Output`] is [`&mut VecDeque`](VecDeque).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `VecDeque::collector_mut()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
#[derive(Debug)]
pub struct CollectorMut<'a, T>(pub(super) &'a mut VecDeque<T>);
