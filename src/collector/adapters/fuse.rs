use std::ops::ControlFlow;

use crate::collector::{Collector, CollectorBase};

/// A collector that can "safely" collect items even after
/// the underlying collector has stopped accumulating,
/// without triggering undesired behaviors.
///
/// This `struct` is created by [`CollectorBase::fuse()`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Fuse<C> {
    collector: C,
    break_hint: ControlFlow<()>,
}

impl<C> Fuse<C>
where
    C: CollectorBase,
{
    #[inline]
    pub(in crate::collector) fn new(collector: C) -> Self {
        Self {
            break_hint: collector.break_hint(),
            collector,
        }
    }
}

impl<C> Fuse<C> {
    /// Use [`Fuse::break_hint()`].
    #[inline]
    #[deprecated(since = "0.4.0", note = "Use `Fuse::break_hint()`")]
    pub fn finished(&self) -> bool {
        self.break_hint.is_break()
    }

    #[inline]
    fn collect_impl(&mut self, f: impl FnOnce(&mut C) -> ControlFlow<()>) -> ControlFlow<()> {
        self.break_hint?;

        self.break_hint = f(&mut self.collector);
        self.break_hint
    }
}

impl<C> CollectorBase for Fuse<C>
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
        self.break_hint
    }
}

impl<C, T> Collector<T> for Fuse<C>
where
    C: Collector<T>,
{
    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.collect_impl(|collector| collector.collect(item))
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.collect_impl(|collector| collector.collect_many(items))
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        if self.break_hint.is_break() {
            self.finish()
        } else {
            self.collector.collect_then_finish(items)
        }
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
        /// We use
        ///
        /// Precondition:
        /// - [`crate::collector::Collector::take_while()`]
        /// - [`crate::vec::IntoCollector`]
        #[test]
        fn all_collect_methods(
            nums in propvec(any::<i32>(), ..=5),
            // We only simulate whether the collector has stopped on construction,
            // or stops later (rely on `take_while()` to stop).
            take_count in prop_oneof![
                1 => Just(0),
                9 => Just(999),
            ],
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
                    .take_while(|&num| num > 0)
                    .fuse()
            },
            should_break_pred: |mut iter| take_count == 0 || !iter.all(|num| num > 0),
            pred: |mut iter, output, remaining| {
                let expected = iter.by_ref().take_while(|&num| num > 0).take(take_count);

                if expected.ne(output) {
                    Err(PredError::IncorrectOutput)
                } else if iter.ne(remaining) {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector_may_fused([1, 2])
    }
}
