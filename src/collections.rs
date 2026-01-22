//! [`Collector`]s for collections in the standard library
//!
//! This module corresponds to [`std::collections`].

pub mod binary_heap;
pub mod btree_map;
pub mod btree_set;
#[cfg(feature = "std")]
// So that doc.rs doesn't put both "std" and "alloc" in feature flag.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod hash_map;
#[cfg(feature = "std")]
// So that doc.rs doesn't put both "std" and "alloc" in feature flag.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod hash_set;
pub mod linked_list;
pub mod vec_deque;

use std::ops::ControlFlow;

use crate::collector::{Collector, CollectorBase, IntoCollector};

#[cfg(feature = "std")]
use std::{
    cmp::Eq,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    hash::{BuildHasher, Hash},
};

#[cfg(not(feature = "std"))]
// Hashtables are not in `alloc`.
use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

#[cfg(feature = "alloc")]
use std::cmp::Ord;

macro_rules! collector_impl {
    (
        $feature:literal, $mod:ident::$coll_name:ident<$($generic:ident),*>, $item_ty:ty,
        $item_pat:pat_param, $push_method_name:ident($($item_args:expr),*),
        $($gen_bound:ident: $bound:path),* $(,)?
    ) => {
        #[cfg(feature = $feature)]
        // So that doc.rs doesn't put both "std" and "alloc" in feature flag.
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<$($generic),*> IntoCollector for $coll_name<$($generic),*>
        where
            $($gen_bound: $bound,)*
        {
            type Output = Self;
            type IntoCollector = $mod::IntoCollector<$($generic),*>;

            #[inline]
            fn into_collector(self) -> Self::IntoCollector {
                $mod::IntoCollector(self)
            }
        }

        #[cfg(feature = $feature)]
        // So that doc.rs doesn't put both "std" and "alloc" in feature flag.
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<'a, $($generic),*> IntoCollector for &'a mut $coll_name<$($generic),*>
        where
            $($gen_bound: $bound,)*
        {
            type Output = Self;
            type IntoCollector = $mod::CollectorMut<'a, $($generic),*>;

            #[inline]
            fn into_collector(self) -> Self::IntoCollector {
                $mod::CollectorMut(self)
            }
        }

        #[cfg(feature = $feature)]
        // So that doc.rs doesn't put both "std" and "alloc" in feature flag.
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<$($generic),*> CollectorBase for $mod::IntoCollector<$($generic),*> {
            type Output = $coll_name<$($generic),*>;

            #[inline]
            fn finish(self) -> Self::Output {
                self.0
            }
        }

        #[cfg(feature = $feature)]
        // So that doc.rs doesn't put both "std" and "alloc" in feature flag.
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<$($generic),*> Collector<$item_ty> for $mod::IntoCollector<$($generic),*>
        where
            $($gen_bound: $bound,)*
        {
            #[inline]
            fn collect(&mut self, $item_pat: $item_ty) -> ControlFlow<()> {
                // It returns a `bool`, so we will return a `ControlFlow` based on it, right?
                // No. `false` is just a signal that "it cannot collect the item at the moment,"
                // not "it cannot collect items from now on."
                self.0.$push_method_name($($item_args),*);
                ControlFlow::Continue(())
            }

            #[inline]
            fn collect_many(&mut self, items: impl IntoIterator<Item = $item_ty>) -> ControlFlow<()> {
                self.0.extend(items);
                ControlFlow::Continue(())
            }

            #[inline]
            fn collect_then_finish(mut self, items: impl IntoIterator<Item = $item_ty>) -> Self::Output {
                self.0.extend(items);
                self.0
            }
        }

        #[cfg(feature = $feature)]
        // So that doc.rs doesn't put both "std" and "alloc" in feature flag.
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<'a, $($generic),*> CollectorBase for $mod::CollectorMut<'a, $($generic),*> {
            type Output = &'a mut $coll_name<$($generic),*>;

            #[inline]
            fn finish(self) -> Self::Output {
                self.0
            }
        }

        #[cfg(feature = $feature)]
        // So that doc.rs doesn't put both "std" and "alloc" in feature flag.
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<'a, $($generic),*> Collector<$item_ty> for $mod::CollectorMut<'a, $($generic),*>
        where
            $($gen_bound: $bound,)*
        {
            #[inline]
            fn collect(&mut self, $item_pat: $item_ty) -> ControlFlow<()> {
                // It returns a `bool`, so we will return a `ControlFlow` based on it, right?
                // No. `false` is just a signal that "it cannot collect the item at the moment,"
                // not "it cannot collect items from now on."
                self.0.$push_method_name($($item_args),*);
                ControlFlow::Continue(())
            }

            #[inline]
            fn collect_many(&mut self, items: impl IntoIterator<Item = $item_ty>) -> ControlFlow<()> {
                self.0.extend(items);
                ControlFlow::Continue(())
            }

            #[inline]
            fn collect_then_finish(self, items: impl IntoIterator<Item = $item_ty>) -> Self::Output {
                self.0.extend(items);
                self.0
            }
        }

        #[cfg(feature = $feature)]
        // So that doc.rs doesn't put both "std" and "alloc" in feature flag.
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<$($generic),*> Default for $mod::IntoCollector<$($generic),*>
        where
            $($gen_bound: $bound,)*
            // Needed because of HashMap and HashSet (they also require S: Default).
            $coll_name<$($generic),*>: Default,
        {
            fn default() -> Self {
                // This is to make sure that we can't construct a default value
                // without it being usable right away as a Collector
                // (e.g. BTreeSet<T> missing T: Ord).
                $coll_name::default().into_collector()
            }
        }
    };
}

collector_impl!(
    "std", hash_map::HashMap<K, V, S>, (K, V),
    (key, value), insert(key, value),
    K: Hash, K: Eq, S: BuildHasher,
);

collector_impl!(
    "std", hash_set::HashSet<T, S>, T,
    item, insert(item),
    T: Hash, T: Eq, S: BuildHasher,
);

collector_impl!(
    "alloc", btree_map::BTreeMap<K, V>, (K, V),
    (key, value), insert(key, value),
    K: Ord,
);

collector_impl!(
    "alloc", btree_set::BTreeSet<T>, T,
    item, insert(item),
    T: Ord,
);

collector_impl!(
    "alloc", binary_heap::BinaryHeap<T>, T,
    item, push(item),
    T: Ord,
);

#[rustfmt::skip]
collector_impl!(
    "alloc", linked_list::LinkedList<T>, T,
    item, push_back(item),
);

#[rustfmt::skip]
collector_impl!(
    "alloc", vec_deque::VecDeque<T>, T,
    item, push_back(item),
);
