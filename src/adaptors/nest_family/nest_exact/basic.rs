use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

use super::{super::strategy::CloneStrategy, with_strategy::WithStrategy};

///
pub struct NestExact<CO, CI>(WithStrategy<CO, CloneStrategy<CI>>)
where
    CI: Collector + Clone;

impl<CO, CI> NestExact<CO, CI>
where
    CI: Collector + Clone,
{
    pub(crate) fn new(outer: CO, inner: CI) -> Self {
        Self(WithStrategy::new(outer, CloneStrategy::new(inner)))
    }
}

impl<CO, CI> Collector for NestExact<CO, CI>
where
    CO: Collector<Item = CI::Output>,
    CI: Collector + Clone,
{
    type Item = CI::Item;

    type Output = CO::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.0.collect(item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0.finish()
    }

    #[inline]
    fn break_hint(&self) -> bool {
        self.0.break_hint()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.0.collect_many(items)
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.0.collect_then_finish(items)
    }
}

impl<CO, CI> RefCollector for NestExact<CO, CI>
where
    CO: Collector<Item = CI::Output>,
    CI: RefCollector + Clone,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.0.collect_ref(item)
    }
}
