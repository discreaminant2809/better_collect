use crate::Collector;
use crate::RefCollector;

use std::ops::ControlFlow;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
impl<T> Collector for Vec<T> {
    type Item = T;

    type Output = Self;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.push(item);
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
}

#[cfg(feature = "alloc")]
impl<T: Copy> RefCollector for Vec<T> {
    type Item = T;

    type Output = Self;

    #[inline]
    fn collect(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.push(*item);
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
}
