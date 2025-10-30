use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

/// Unit type will always stop collecting, no matter the type it collects.
impl<T> Collector<T> for () {
    type Output = ();

    /// Unit type will always stop collecting, no matter the type it collects.
    #[inline]
    fn collect(&mut self, _item: T) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    #[inline]
    fn finish(self) -> Self::Output {}

    /// It won't consume any items in an iterator, either.
    #[inline]
    fn collect_many(&mut self, _items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    /// It won't consume any items in an iterator, either.
    #[inline]
    fn collect_then_finish(self, _items: impl IntoIterator<Item = T>) -> Self::Output {}
}

impl<T> RefCollector<T> for () {
    #[inline]
    fn collect_ref(&mut self, _item: &mut T) -> ControlFlow<()> {
        ControlFlow::Break(())
    }
}
