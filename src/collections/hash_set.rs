//! [`Collector`]s for [`HashSet`]
//!
//! This module corresponds to [`std::collections::hash_set`].
//!
//! [`Collector`]: crate::collector::Collector

use std::collections::HashSet;

/// A [`Collector`] that inserts collected items into a [`HashSet`].
/// Its [`Output`] is [`HashSet`].
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `HashSet::into_collector()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
pub struct IntoCollector<T, S>(pub(super) HashSet<T, S>);

/// A [`Collector`] that inserts collected items into a [`&mut HashSet`](HashSet).
/// Its [`Output`] is [`&mut HashSet`](HashSet).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `HashSet::collector_mut()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
pub struct CollectorMut<'a, T, S>(pub(super) &'a mut HashSet<T, S>);
