use crate::{Collector, RefCollector};

pub struct Owned<C: RefCollector> {
    collector: C,
}

impl<C: RefCollector> Owned<C> {
    pub(crate) fn new(collector: C) -> Self {
        Self { collector }
    }
}

impl<C: RefCollector> Collector for Owned<C> {
    type Item = C::Item;

    type Output = C::Output;

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> core::ops::ControlFlow<()> {
        self.collector.collect(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    #[inline]
    fn collect_many(
        &mut self,
        items: impl IntoIterator<Item = Self::Item>,
    ) -> core::ops::ControlFlow<()> {
        self.collector.collect_many(items)
    }
}
