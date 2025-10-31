use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

/// Unit type will always stop collecting, no matter the type it collects.
impl Collector for () {
    type Item = ();
    type Output = ();

    /// Unit type will always stop collecting, no matter the type it collects.
    #[inline]
    fn collect(&mut self, _item: Self::Item) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    #[inline]
    fn finish(self) -> Self::Output {}

    /// It won't consume any items in an iterator, either.
    #[inline]
    fn collect_many(&mut self, _items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    /// It won't consume any items in an iterator, either.
    #[inline]
    fn collect_then_finish(self, _items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        // Nothing worth doing here
    }
}

impl RefCollector for () {
    #[inline]
    fn collect_ref(&mut self, _item: &mut Self::Item) -> ControlFlow<()> {
        ControlFlow::Break(())
    }
}
