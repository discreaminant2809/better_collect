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

use crate::collector::{Collector, IntoCollector, RefCollector};

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
            type Item = $item_ty;
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
            type Item = $item_ty;
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
        impl<$($generic),*> Collector for $mod::IntoCollector<$($generic),*>
        where
            $($gen_bound: $bound,)*
        {
            type Item = $item_ty;
            type Output = $coll_name<$($generic),*>;

            #[inline]
            fn collect(&mut self, $item_pat: Self::Item) -> ControlFlow<()> {
                // It returns a `bool`, so we will return a `ControlFlow` based on it, right?
                // No. `false` is just a signal that "it cannot collect the item at the moment,"
                // not "it cannot collect items from now on."
                self.0.$push_method_name($($item_args),*);
                ControlFlow::Continue(())
            }

            #[inline]
            fn finish(self) -> Self::Output {
                self.0
            }

            #[inline]
            fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
                self.0.extend(items);
                ControlFlow::Continue(())
            }

            #[inline]
            fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
                self.0.extend(items);
                self.0
            }
        }

        #[cfg(feature = $feature)]
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<$($generic),*> RefCollector for $mod::IntoCollector<$($generic),*>
        where
            $($gen_bound: $bound,)*
            Self::Item: Copy,
        {
            #[inline]
            fn collect_ref(&mut self, &mut item: &mut Self::Item) -> ControlFlow<()> {
                self.collect(item)
            }
        }

        #[cfg(feature = $feature)]
        // So that doc.rs doesn't put both "std" and "alloc" in feature flag.
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<'a, $($generic),*> Collector for $mod::CollectorMut<'a, $($generic),*>
        where
            $($gen_bound: $bound,)*
        {
            type Item = $item_ty;
            type Output = &'a mut $coll_name<$($generic),*>;

            #[inline]
            fn collect(&mut self, $item_pat: Self::Item) -> ControlFlow<()> {
                // It returns a `bool`, so we will return a `ControlFlow` based on it, right?
                // No. `false` is just a signal that "it cannot collect the item at the moment,"
                // not "it cannot collect items from now on."
                self.0.$push_method_name($($item_args),*);
                ControlFlow::Continue(())
            }

            #[inline]
            fn finish(self) -> Self::Output {
                self.0
            }

            #[inline]
            fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
                self.0.extend(items);
                ControlFlow::Continue(())
            }

            #[inline]
            fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
                self.0.extend(items);
                self.0
            }
        }

        #[cfg(feature = $feature)]
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<$($generic),*> RefCollector for $mod::CollectorMut<'_, $($generic),*>
        where
            $($gen_bound: $bound,)*
            Self::Item: Copy,
        {
            #[inline]
            fn collect_ref(&mut self, &mut item: &mut Self::Item) -> ControlFlow<()> {
                self.collect(item)
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
