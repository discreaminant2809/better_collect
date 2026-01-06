//! [`Collector`]s for [`LinkedList`]
//!
//! This module corresponds to [`std::collections::linked_list`].
//!
//! [`Collector`]: crate::collector::Collector

#[cfg(not(feature = "std"))]
use alloc::collections::LinkedList;
#[cfg(feature = "std")]
use std::collections::LinkedList;

/// A [`Collector`] that pushes collected items into the back of a [`LinkedList`].
/// Its [`Output`] is [`LinkedList`].
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `LinkedList::into_collector()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
#[derive(Debug, Clone)]
pub struct IntoCollector<T>(pub(super) LinkedList<T>);

/// A [`Collector`] that pushes collected items into the back of a [`&mut LinkedList`](LinkedList).
/// Its [`Output`] is [`&mut LinkedList`](LinkedList).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `LinkedList::collector_mut()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
#[derive(Debug)]
pub struct CollectorMut<'a, T>(pub(super) &'a mut LinkedList<T>);
