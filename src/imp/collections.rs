use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

#[cfg(feature = "std")]
use std::{
    cmp::Eq,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    hash::{BuildHasher, Hash},
};

#[cfg(all(feature = "alloc", not(feature = "std")))]
// Hashtables are not in `alloc`.
use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

#[cfg(feature = "alloc")]
use std::cmp::Ord;

macro_rules! collection_impl {
    (
        $feature:literal, $name:ident<$($generic:ident),*>, $item_ty:ty,
        $item_pat:pat_param, $push_method_name:ident($($item_args:expr),*),
        $($gen_bound:ident: $bound:path),* $(,)?
    ) => {
        #[cfg(feature = $feature)]
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<$($generic),*> Collector for $name<$($generic),*>
        where
            $($gen_bound: $bound,)*
        {
            type Item = $item_ty;
            type Output = Self;

            #[inline]
            fn collect(&mut self, $item_pat: Self::Item) -> ControlFlow<()> {
                // It returns a `bool`, so we will return a `ControlFlow` based on it, right?
                // No. `false` is just a signal that "it cannot collect the item at the moment,"
                // not "it cannot collect items from now on."
                self.$push_method_name($($item_args),*);
                ControlFlow::Continue(())
            }

            #[inline]
            fn finish(self) -> Self::Output {
                self
            }

            #[inline]
            fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
                self.extend(items);
                ControlFlow::Continue(())
            }

            #[inline]
            fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
                self.extend(items);
                self
            }
        }

        #[cfg(feature = $feature)]
        #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
        impl<$($generic),*> RefCollector for $name<$($generic),*>
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

collection_impl!(
    "std", HashSet<T, S>, T,
    item, insert(item),
    T: Hash, T: Eq, S: BuildHasher,
);

collection_impl!(
    "std", HashMap<K, V, S>, (K, V),
    (key, value), insert(key, value),
    K: Hash, K: Eq, S: BuildHasher,
);

collection_impl!(
    "alloc", BTreeSet<T>, T,
    item, insert(item),
    T: Ord,
);

collection_impl!(
    "alloc", BTreeMap<K, V>, (K, V),
    (key, value), insert(key, value),
    K: Ord,
);

collection_impl!(
    "alloc", BinaryHeap<T>, T,
    item, push(item),
    T: Ord,
);

collection_impl!("alloc", LinkedList<T>, T, item, push_back(item),);

collection_impl!("alloc", VecDeque<T>, T, item, push_back(item),);
