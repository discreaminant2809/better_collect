use crate::collector::{Collector, RefCollector};

use std::{fmt::Debug, ops::ControlFlow};

/// A [`Collector`] that uses a closure to determine whether an item should be collected.
///
/// This `struct` is created by [`Collector::filter()`]. See its documentation for more.
#[derive(Clone)]
pub struct Filter<C, F> {
    collector: C,
    pred: F,
}

impl<C, F> Filter<C, F> {
    pub(in crate::collector) fn new(collector: C, pred: F) -> Self {
        Self { collector, pred }
    }
}

impl<C, F> Collector for Filter<C, F>
where
    C: Collector,
    F: FnMut(&C::Item) -> bool,
{
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
    fn break_hint(&self) -> bool {
        self.collector.break_hint()
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

impl<C, F> RefCollector for Filter<C, F>
where
    C: RefCollector,
    F: FnMut(&C::Item) -> bool,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        if (self.pred)(item) {
            self.collector.collect_ref(item)
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl<C: Debug, F> Debug for Filter<C, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Filter")
            .field("collector", &self.collector)
            .finish()
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use proptest::collection::vec as propvec;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

    use crate::prelude::*;
    use crate::test_utils::{BasicCollectorTester, CollectorTesterExt, PredError};

    proptest! {
        /// Precondition:
        /// - [`crate::collector::Collector::take()`]
        /// - [`crate::vec::IntoCollector`]
        #[test]
        fn all_collect_methods(
            nums in propvec(any::<i32>(), ..=8),
            take_count in ..=8_usize,
        ) {
            all_collect_methods_impl(nums, take_count)?;
        }
    }

    fn all_collect_methods_impl(nums: Vec<i32>, take_count: usize) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || nums.iter().copied(),
            collector_factory: || {
                vec![]
                    .into_collector()
                    .take(take_count)
                    .filter(|&num| num >= 0)
            },
            should_break_pred: |iter| iter.filter(|&num| num >= 0).count() >= take_count,
            pred: |mut iter, output, remaining| {
                let expected = iter.by_ref().filter(|&num| num >= 0).take(take_count);

                if expected.ne(output) {
                    Err(PredError::IncorrectOutput)
                } else if iter.ne(remaining) {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_ref_collector()
    }
}
