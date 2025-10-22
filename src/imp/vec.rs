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
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }

    #[inline]
    fn reserve(&mut self, additional_min: usize, _additional_max: Option<usize>) {
        self.reserve(additional_min);
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

#[cfg(feature = "alloc")]
impl<T: Copy> RefCollector for Vec<T> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.push(*item);
        ControlFlow::Continue(())
    }
}
