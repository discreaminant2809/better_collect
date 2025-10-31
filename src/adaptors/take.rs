use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

#[derive(Debug, Clone)]
pub struct Take<C> {
    collector: C,
    remaining: usize,
}

impl<C> Take<C> {
    pub(crate) fn new(collector: C, n: usize) -> Self {
        Self {
            collector,
            remaining: n,
        }
    }

    #[inline]
    fn collect_impl(&mut self, f: impl FnOnce(&mut C) -> ControlFlow<()>) -> ControlFlow<()> {
        if self.remaining == 0 {
            return ControlFlow::Break(());
        }

        self.remaining -= 1;
        let cf = f(&mut self.collector);

        if self.remaining == 0 {
            ControlFlow::Break(())
        } else {
            cf
        }
    }
}

impl<C: Collector> Collector for Take<C> {
    type Item = C::Item;
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.collect_impl(|collector| collector.collect(item))
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     let (lower, upper) = self.collector.size_hint();
    //     (
    //         lower.min(self.remaining),
    //         upper.map(|u| u.min(self.remaining)),
    //     )
    // }

    // fn reserve(&mut self, mut additional_min: usize, mut additional_max: Option<usize>) {
    //     additional_min = additional_min.min(self.remaining);
    //     additional_max = Some(additional_max.map_or(self.remaining, |additional_max| {
    //         additional_max.min(self.remaining)
    //     }));

    //     self.collector.reserve(additional_min, additional_max);
    // }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // FIXME: consider the iterator's size hint to (partly) avoid `inspect`.
        self.collector.collect_many(
            items
                .into_iter()
                .take(self.remaining)
                // Since the collector may not collect all `remaining` items
                .inspect(|_| self.remaining -= 1),
        )
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        // No need to track the state anymore - we'll be gone!
        self.collector
            .collect_then_finish(items.into_iter().take(self.remaining))
    }
}

impl<C: RefCollector> RefCollector for Take<C> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.collect_impl(|collector| collector.collect_ref(item))
    }
}
