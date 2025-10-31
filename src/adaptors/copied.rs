use crate::{Collector, RefCollector};

use std::ops::ControlFlow;

#[derive(Debug, Clone)]
pub struct Copied<C>(C);

impl<C> Copied<C> {
    pub(crate) fn new(collector: C) -> Self {
        Self(collector)
    }
}

impl<C> Collector for Copied<C>
where
    C: Collector,
{
    type Item = C::Item;
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.0.collect(item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0.finish()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.0.collect_many(items)
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.0.collect_then_finish(items)
    }
}

impl<C> RefCollector for Copied<C>
where
    Self::Item: Copy,
    C: Collector,
{
    #[inline]
    fn collect_ref(&mut self, &mut item: &mut Self::Item) -> ControlFlow<()> {
        self.0.collect(item)
    }
}
