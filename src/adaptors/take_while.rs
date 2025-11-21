use std::{fmt::Debug, ops::ControlFlow};

use crate::{Collector, RefCollector};

/// A [`Collector`] that accumulates items as long as a predicate returns `true`.
///
/// This `struct` is created by [`Collector::take_while()`]. See its documentation for more.
#[derive(Clone)]
pub struct TakeWhile<C, F> {
    collector: C,
    pred: F,
}

impl<C, F> TakeWhile<C, F> {
    pub(crate) fn new(collector: C, pred: F) -> Self {
        Self { collector, pred }
    }
}

impl<C, F> Collector for TakeWhile<C, F>
where
    C: Collector,
    F: FnMut(&C::Item) -> bool,
{
    type Item = C::Item;

    type Output = C::Output;

    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        if (self.pred)(&item) {
            self.collector.collect(item)
        } else {
            ControlFlow::Break(())
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // Be careful - the underlying collector may stop before the predicate return false.
        let mut all_true = true;
        let cf = self
            .collector
            .collect_many(items.into_iter().take_while(|item| {
                // We trust the implementation of the standard library and the collector.
                // They should short-circuit on the first false.
                all_true = (self.pred)(item);
                all_true
            }));

        if all_true { cf } else { ControlFlow::Break(()) }
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().take_while(self.pred))
    }
}

impl<C, F> RefCollector for TakeWhile<C, F>
where
    C: RefCollector,
    F: FnMut(&C::Item) -> bool,
{
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        if (self.pred)(item) {
            self.collector.collect_ref(item)
        } else {
            ControlFlow::Break(())
        }
    }
}

impl<C: Debug, F> Debug for TakeWhile<C, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TakeWhile")
            .field("collector", &self.collector)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use proptest::collection::vec as propvec;
    use proptest::prelude::*;

    use crate::{Collector, IntoCollector};

    proptest! {
        #[test]
        fn collect_many(
            nums in propvec(any::<i32>(), ..100),
        ) {
            let (collector_way, iter_way) = collect_many_helper(nums);
            prop_assert_eq!(collector_way, iter_way);
        }
    }

    fn collect_many_helper(nums: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        let iter1 = nums.iter().copied();
        let iter2 = iter1.clone().take_while(|&num| num % 4 == 0);

        let mut collector = vec![].into_collector().take_while(|&num| num % 4 == 0);
        let _ = collector.collect_many(iter1);
        let collector_way = collector.finish();

        let iter_way = iter2.collect();

        (collector_way, iter_way)
    }
}
