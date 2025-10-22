use crate::{Collector, RefCollector};

use std::ops::ControlFlow;

pub struct Filter<C, F> {
    collector: C,
    pred: F,
}

impl<C, F> Filter<C, F> {
    pub(crate) fn new(collector: C, pred: F) -> Self {
        Self { collector, pred }
    }
}

impl<C: Collector, F: FnMut(&C::Item) -> bool> Collector for Filter<C, F> {
    type Item = C::Item;

    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        if (self.pred)(&item) {
            self.collector.collect(item)
        } else {
            ControlFlow::Continue(())
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // See: https://doc.rust-lang.org/1.90.0/src/core/iter/adapters/filter.rs.html#117-121
        (0, self.collector.size_hint().1)
    }

    #[inline]
    fn reserve(&mut self, _additional_min: usize, additional_max: Option<usize>) {
        // See: https://doc.rust-lang.org/1.90.0/src/core/iter/adapters/filter.rs.html#117-121
        self.collector.reserve(0, additional_max);
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.collector
            .collect_many(items.into_iter().filter(&mut self.pred))
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().filter(self.pred))
    }
}

impl<C: RefCollector, F: FnMut(&C::Item) -> bool> RefCollector for Filter<C, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        if (self.pred)(item) {
            self.collector.collect_ref(item)
        } else {
            ControlFlow::Continue(())
        }
    }
}
