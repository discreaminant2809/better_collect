//! [`Collector`]s for [`BTreeSet`]
//!
//! This module corresponds to [`std::collections::btree_set`].
//!
//! [`Collector`]: crate::collector::Collector

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeSet;
#[cfg(feature = "std")]
use std::collections::BTreeSet;

/// A [`Collector`] that inserts collected items into a [`BTreeSet`].
/// Its [`Output`] is [`BTreeSet`].
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `BTreeSet::into_collector()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
pub struct IntoCollector<T>(pub(super) BTreeSet<T>);

/// A [`Collector`] that inserts collected items into a [`&mut BTreeSet`](BTreeSet).
/// Its [`Output`] is [`&mut BTreeSet`](BTreeSet).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `BTreeSet::collector_mut()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
pub struct CollectorMut<'a, T>(pub(super) &'a mut BTreeSet<T>);
