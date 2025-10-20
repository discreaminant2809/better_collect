use crate::{Collector, RefCollector};

use std::ops::ControlFlow;

pub struct Cloned<C>(C);

impl<C> Cloned<C> {
    pub fn new(collector: C) -> Self {
        Self(collector)
    }
}

impl<C: Collector<Item: Clone>> RefCollector for Cloned<C> {
    type Item = C::Item;

    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.0.collect(item.clone())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0.finish()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.0.collect_many(items)
    }
}
