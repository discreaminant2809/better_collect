#[cfg(feature = "std")]
use std::collections::HashSet;
use std::{
    cmp::Eq,
    hash::{BuildHasher, Hash},
    ops::ControlFlow,
};

use crate::Collector;
#[cfg(feature = "std")]
use crate::RefCollector;

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T, S> Collector<T> for HashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Output = Self;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        // It returns a `bool`, so we will return a `ControlFlow` based on it, right?
        // No. `false` is just a signal that "it cannot collect the item at the moment,"
        // not "it cannot collect items from now on."
        self.insert(item);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.extend(items);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.extend(items);
        self
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T, S> RefCollector<T> for HashSet<T, S>
where
    T: Copy + Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    fn collect_ref(&mut self, &mut item: &mut T) -> ControlFlow<()> {
        self.collect(item)
    }
}
