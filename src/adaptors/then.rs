use std::ops::ControlFlow;

use super::Fuse;
use crate::{Collector, RefCollector};

pub struct Then<C1, C2> {
    collector1: Fuse<C1>,
    collector2: C2,
}

impl<C1, C2> Then<C1, C2> {
    pub(crate) fn new(collector1: C1, collector2: C2) -> Self {
        Self {
            collector1: Fuse::new(collector1),
            collector2,
        }
    }
}

impl<C1: RefCollector, C2: Collector<Item = C1::Item>> Collector for Then<C1, C2> {
    type Item = C1::Item;

    type Output = (C1::Output, C2::Output);

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        match (
            self.collector1.collect_ref(&mut item),
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

    fn reserve(&mut self, additional_min: usize, additional_max: Option<usize>) {
        let (lower1, upper1) = self.collector1.size_hint();

        // Both have the same theme: the 2nd collector reserves the left-over amount.
        let (reserve_lower1, reserve_lower2) = if additional_min > lower1 {
            (lower1, additional_min - lower1)
        } else {
            (additional_min, 0)
        };

        let (reserve_upper1, reserve_upper2) = match (additional_max, upper1) {
            (Some(additional_max), Some(upper1)) if additional_max > upper1 => {
                (Some(upper1), Some(additional_max - upper1))
            }
            (additional_max, _) => (additional_max, Some(0)),
        };

        self.collector1.reserve(reserve_lower1, reserve_upper1);
        self.collector2.reserve(reserve_lower2, reserve_upper2);
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower1, upper1) = self.collector1.size_hint();
        let (lower2, upper2) = self.collector2.size_hint();

        (
            lower1.saturating_add(lower2),
            (|| upper1?.checked_add(upper2?))(),
        )
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        while let Some(mut item) = items.next() {
            if self.collector1.collect_ref(&mut item).is_break() {
                return self.collector2.collect_many(items);
            }

            if self.collector2.collect(item).is_break() {
                return self.collector1.collect_many(items);
            }
        }

        // If any of them breaks, the `return` path will've been taken instead.
        ControlFlow::Continue(())
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        let mut items = items.into_iter();

        while let Some(mut item) = items.next() {
            if self.collector1.collect_ref(&mut item).is_break() {
                return (
                    self.collector1.finish(),
                    self.collector2.collect_then_finish(items),
                );
            }

            if self.collector2.collect(item).is_break() {
                return (
                    self.collector1.collect_then_finish(items),
                    self.collector2.finish(),
                );
            }
        }

        self.finish()
    }
}

impl<C1: RefCollector, C2: RefCollector<Item = C1::Item>> RefCollector for Then<C1, C2> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        match (
            self.collector1.collect_ref(item),
            self.collector2.collect_ref(item),
        ) {
            (ControlFlow::Break(_), ControlFlow::Break(_)) => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }
}
