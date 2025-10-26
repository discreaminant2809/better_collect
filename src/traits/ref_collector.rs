use std::ops::ControlFlow;

use crate::{Chain, Collector, Funnel, Then, assert_collector, assert_ref_collector};

pub trait RefCollector: Collector {
    /// Returns a [`ControlFlow`] to command whether to stop the collection.
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()>;

    #[inline]
    fn then<C: Collector<Item = Self::Item>>(self, other: C) -> Then<Self, C> {
        assert_collector(Then::new(self, other))
    }

    #[inline]
    fn chain<C: Collector<Item = Self::Item>>(self, other: C) -> Chain<Self, C> {
        assert_collector(Chain::new(self, other))
    }

    #[inline]
    fn funnel<E, F>(self, func: F) -> Funnel<Self, E, F>
    where
        F: FnMut(&mut E) -> &mut Self::Item,
    {
        assert_ref_collector(Funnel::new(self, func))
    }
}
