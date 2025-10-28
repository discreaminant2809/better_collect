use std::ops::ControlFlow;

use crate::{Chain, Collector, Funnel, Then, assert_collector, assert_ref_collector};

pub trait RefCollector<T>: Collector<T> {
    /// Returns a [`ControlFlow`] to command whether to stop the collection.
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()>;

    #[inline]
    fn then<C>(self, other: C) -> Then<Self, C>
    where
        C: Collector<T>,
    {
        assert_collector(Then::new(self, other))
    }

    #[inline]
    fn chain<C>(self, other: C) -> Chain<Self, C>
    where
        C: Collector<T>,
    {
        assert_collector(Chain::new(self, other))
    }

    #[inline]
    fn funnel<F, U>(self, func: F) -> Funnel<Self, U, F>
    where
        F: FnMut(&mut U) -> &mut T,
    {
        assert_ref_collector(Funnel::new(self, func))
    }
}
