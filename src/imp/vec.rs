use crate::Collector;
use crate::RefCollector;

use std::ops::ControlFlow;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T> Collector<T> for Vec<T> {
    type Output = Self;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.push(item);
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

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T: Copy> RefCollector<T> for Vec<T> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        self.push(*item);
        ControlFlow::Continue(())
    }
}
