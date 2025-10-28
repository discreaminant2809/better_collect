use crate::{Collector, RefCollector};

use std::ops::ControlFlow;

pub struct Cloned<C>(C);

impl<C> Cloned<C> {
    pub(crate) fn new(collector: C) -> Self {
        Self(collector)
    }
}

impl<T, C> Collector<T> for Cloned<C>
where
    C: Collector<T>,
{
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.0.collect(item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0.finish()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.0.collect_many(items)
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.0.collect_then_finish(items)
    }
}

impl<T, C> RefCollector<T> for Cloned<C>
where
    T: Clone,
    C: Collector<T>,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        self.0.collect(item.clone())
    }
}
