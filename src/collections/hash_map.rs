//! [`Collector`]s for [`HashMap`]
//!
//! This module corresponds to [`std::collections::hash_map`].
//!
//! [`Collector`]: crate::collector::Collector

use std::collections::HashMap;
// #[cfg(feature = "unstable")]
// use std::{
//     collections::hash_map::{Entry, OccupiedEntry, VacantEntry},
//     hash::Hash,
// };

// #[cfg(feature = "unstable")]
// use crate::aggregate::{Group, GroupMap, OccupiedGroup, VacantGroup};

/// A [`Collector`] that inserts collected items into a [`HashMap`].
/// Its [`Output`] is [`HashMap`].
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `HashMap::into_collector()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
#[derive(Debug, Clone)]
pub struct IntoCollector<K, V, S>(pub(super) HashMap<K, V, S>);

/// A [`Collector`] that inserts collected items into a [`&mut HashMap`](HashMap).
/// Its [`Output`] is [`&mut HashMap`](HashMap).
///
/// This also implements [`RefCollector`] if `T` is [`Copy`].
///
/// This struct is created by `HashMap::collector_mut()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
/// [`RefCollector`]: crate::collector::RefCollector
#[derive(Debug)]
pub struct CollectorMut<'a, K, V, S>(pub(super) &'a mut HashMap<K, V, S>);

// #[cfg(feature = "unstable")]
// // #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "unstable"))))]
// impl<'a, K, V> VacantGroup for VacantEntry<'a, K, V> {
//     type Key = K;

//     type Value = V;

//     #[inline]
//     fn key(&self) -> &Self::Key {
//         self.key()
//     }

//     #[inline]
//     fn insert(self, value: Self::Value) {
//         self.insert(value);
//     }
// }

// #[cfg(feature = "unstable")]
// // #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "unstable"))))]
// impl<'a, K, V> OccupiedGroup for OccupiedEntry<'a, K, V> {
//     type Key = K;

//     type Value = V;

//     #[inline]
//     fn key(&self) -> &Self::Key {
//         self.key()
//     }

//     #[inline]
//     fn value(&self) -> &Self::Value {
//         self.get()
//     }

//     #[inline]
//     fn value_mut(&mut self) -> &mut Self::Value {
//         self.get_mut()
//     }
// }

// #[cfg(feature = "unstable")]
// // #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "unstable"))))]
// impl<K, V> GroupMap for HashMap<K, V>
// where
//     K: Eq + Hash,
// {
//     type Key = K;

//     type Value = V;

//     type Vacant<'a>
//         = VacantEntry<'a, K, V>
//     where
//         Self: 'a;

//     type Occupied<'a>
//         = OccupiedEntry<'a, K, V>
//     where
//         Self: 'a;

//     #[inline]
//     fn group(&mut self, key: Self::Key) -> Group<Self::Occupied<'_>, Self::Vacant<'_>> {
//         match self.entry(key) {
//             Entry::Occupied(entry) => Group::Occupied(entry),
//             Entry::Vacant(entry) => Group::Vacant(entry),
//         }
//     }
// }
