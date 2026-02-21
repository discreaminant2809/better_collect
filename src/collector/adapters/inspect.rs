use std::{fmt::Debug, ops::ControlFlow};

use crate::collector::{Collector, CollectorBase};

/// A collector that calls a closure on each item before collecting.
///
/// This `struct` is created by [`CollectorBase::inspect()`]. See its documentation for more.
pub struct Inspect<C, F> {
    collector: C,
    f: F,
}

impl<C, F> Inspect<C, F> {
    pub(in crate::collector) fn new(collector: C, f: F) -> Self {
        Self { collector, f }
    }
}

impl<C, F> CollectorBase for Inspect<C, F>
where
    C: CollectorBase,
{
    type Output = C::Output;

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    #[inline]
    fn break_hint(&self) -> ControlFlow<()> {
        self.collector.break_hint()
    }
}

impl<C, T, F> Collector<T> for Inspect<C, F>
where
    C: Collector<T>,
    F: FnMut(&T),
{
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        (self.f)(&item);
        self.collector.collect(item)
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.collector
            .collect_many(items.into_iter().inspect(&mut self.f))
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().inspect(self.f))
    }
}

impl<C: Debug, F> Debug for Inspect<C, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inspect")
            .field("collector", &self.collector)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use std::cell::Cell;

    use proptest::collection::vec as propvec;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

    use crate::prelude::*;
    use crate::test_utils::{BasicCollectorTester, CollectorTesterExt, PredError};

    // We need to use `take()` to simulate the break case when enough items are skipped.
    // Precondition:
    // - `Vec::IntoCollector`
    // - `Collector::take()`
    // - `Sink`
    proptest! {
        #[test]
        fn all_collect_methods(
            // We keep just enough "space" for the take count to land on
            // each size hint interval.
            // The "diagram" is as below (E = when the take count is equal to either lower or upper bound)
            // 0 1 2 E 4 5 6 E 8 9
            nums in propvec(any::<i32>(), ..=5),
            take_count in ..=5_usize,
        ) {
            all_collect_methods_impl(nums, take_count)?;
        }
    }

    fn all_collect_methods_impl(nums: Vec<i32>, take_count: usize) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || nums.iter().map(|&num| Cell::new(num)),
            collector_factory: || {
                vec![]
                    .into_collector()
                    .take(take_count)
                    .inspect(|num: &Cell<_>| num.update(|x| x + 1))
            },
            should_break_pred: |_| nums.len() >= take_count,
            pred: |mut iter, output, remaining| {
                if iter
                    .by_ref()
                    .inspect(|num| num.update(|x| x + 1))
                    .take(take_count)
                    .ne(output)
                {
                    Err(PredError::IncorrectOutput)
                } else if iter.ne(remaining) {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector()
    }
}
