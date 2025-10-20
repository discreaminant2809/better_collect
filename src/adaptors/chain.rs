use core::ops::ControlFlow;

use super::FuseByRef;
use crate::{Collector, RefCollector};

pub struct Chain<C1: RefCollector, C2> {
    collector1: FuseByRef<C1>,
    collector2: C2,
}

impl<C1: RefCollector, C2> Chain<C1, C2> {
    pub(crate) fn new(collector1: C1, collector2: C2) -> Self {
        Self {
            collector1: FuseByRef::new(collector1),
            collector2,
        }
    }
}

impl<C1: RefCollector, C2: RefCollector<Item = C1::Item>> RefCollector for Chain<C1, C2> {
    type Item = C1::Item;

    type Output = (C1::Output, C2::Output);

    #[inline]
    fn collect(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        match (self.collector1.collect(item), self.collector2.collect(item)) {
            (ControlFlow::Break(_), ControlFlow::Break(_)) => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        (self.collector1.finish(), self.collector2.finish())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        while let Some(mut item) = items.next() {
            if self.collector1.collect(&mut item).is_break() {
                return self.collector2.collect_many(items);
            }

            if self.collector2.collect(&mut item).is_break() {
                return self.collector1.collect_many(items);
            }
        }

        // If any of them breaks, the `return` path will've been taken instead.
        ControlFlow::Continue(())
    }
}

impl<CR: RefCollector, C: Collector<Item = CR::Item>> Collector for Chain<CR, C> {
    type Item = C::Item;

    type Output = (CR::Output, C::Output);

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        match (
            self.collector1.collect(&mut item),
            self.collector2.collect(item),
        ) {
            (ControlFlow::Break(_), ControlFlow::Break(_)) => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        (self.collector1.finish(), self.collector2.finish())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        while let Some(mut item) = items.next() {
            if self.collector1.collect(&mut item).is_break() {
                return self.collector2.collect_many(items);
            }

            if self.collector2.collect(item).is_break() {
                return self.collector1.collect_many(items);
            }
        }

        // If any of them breaks, the `return` path will've been taken instead.
        ControlFlow::Continue(())
    }
}
