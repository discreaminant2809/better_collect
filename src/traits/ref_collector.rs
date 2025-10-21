use std::ops::ControlFlow;

use crate::{Collector, Then, assert_collector};

pub trait RefCollector: Collector {
    /// Returns a [`ControlFlow`] to command whether to stop the collection.
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()>;

    #[inline]
    fn then<C: Collector<Item = Self::Item>>(self, other: C) -> Then<Self, C> {
        assert_collector(Then::new(self, other))
    }

    // #[inline]
    // fn owned(self) -> Owned<Self> {
    //     assert_collector(Owned::new(self))
    // }
}
