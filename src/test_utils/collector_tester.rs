use std::fmt::Debug;

use proptest::{prelude::*, test_runner::TestCaseResult};

use crate::collector::Collector;

/// Test helper that returns parts needed for collector proptest.
///
/// # Notes
///
/// The [`Output`] should be reset for every call. May not needed
/// if you can make the output consistent without resetting.
///
/// [`Output`]: CollectorTester::Output
pub trait CollectorTester {
    type Item;

    type Output<'a>;

    #[allow(clippy::type_complexity)] // Can't satisfy it so I suppress it.
    fn collector_test_parts(
        &mut self,
    ) -> CollectorTestParts<
        impl Iterator<Item = Self::Item> + Clone,
        impl Collector<Self::Item, Output = Self::Output<'_>>,
        impl FnMut(Self::Output<'_>, &mut dyn Iterator<Item = Self::Item>) -> Result<(), PredError>,
    >;
}

/// Test parts for collector testing.
pub struct CollectorTestParts<I, C, P>
where
    I: Iterator + Clone,
    C: Collector<I::Item>,
    P: FnMut(C::Output, &mut dyn Iterator<Item = I::Item>) -> Result<(), PredError>,
{
    /// Iterator provided to feed the collector.
    pub iter: I,
    /// Collector to be tested.
    pub collector: C,
    /// Determines whether the collector should have stopped accumulating
    /// after operation.
    pub should_break: bool,
    /// Predicate on the following being satisfied:
    /// - Output of the collector.
    /// - Remaining of the iterator after the operation.
    pub pred: P,
}

/// An error returned when the collection operations of the collector are not satisfied.
#[derive(Debug)]
pub enum PredError {
    /// Incorrect [`Output`] produced by the collector
    ///
    /// [`Output`]: crate::collector::Collector::Output
    IncorrectOutput,
    /// The [`Iterator`] is not consumed as expected.
    IncorrectIterConsumption,
}

impl PredError {
    fn of_method(self, name: &'static str) -> OfMethod {
        OfMethod {
            name,
            pred_error: self,
        }
    }
}

/// Helper to convert [`PredError`] into [`TestCaseError`].
struct OfMethod {
    name: &'static str,
    pred_error: PredError,
}

impl From<OfMethod> for TestCaseError {
    fn from(OfMethod { name, pred_error }: OfMethod) -> Self {
        Self::Fail(format!("`{name}()` is implemented incorrectly: {pred_error:?}").into())
    }
}

/// Used because we don't want the user to override any methods here.
pub trait CollectorTesterExt: CollectorTester {
    fn test_collector(&mut self) -> TestCaseResult {
        self.test_collector_may_fused(false)
    }

    fn test_collector_may_fused(&mut self, collector_fused: bool) -> TestCaseResult {
        test_collector_part(self, collector_fused)
    }
}

impl<T> CollectorTesterExt for T where T: CollectorTester {}

/// Basic implementation for [`CollectorTester`] for most use case.
/// Opt-out if you test the `collector(_mut)` variant, or the collector and output
/// that may borrow from the tester, or the output is judged differently.
pub struct BasicCollectorTester<ItFac, ClFac, SbPred, Pred, I, C>
// `where` bound is needed otherwise we get "type annotation needed" for the input iterator.
where
    I: Iterator,
    C: Collector<I::Item>,
    ItFac: FnMut() -> I,
    ClFac: FnMut() -> C,
    SbPred: FnMut(I) -> bool,
    Pred: FnMut(I, C::Output, &mut dyn Iterator<Item = I::Item>) -> Result<(), PredError>,
{
    pub iter_factory: ItFac,
    pub collector_factory: ClFac,
    pub should_break_pred: SbPred,
    pub pred: Pred,
}

impl<ItFac, ClFac, SbPred, Pred, I, C> CollectorTester
    for BasicCollectorTester<ItFac, ClFac, SbPred, Pred, I, C>
where
    I: Iterator + Clone,
    C: Collector<I::Item>,
    ItFac: FnMut() -> I,
    ClFac: FnMut() -> C,
    SbPred: FnMut(I) -> bool,
    Pred: FnMut(I, C::Output, &mut dyn Iterator<Item = I::Item>) -> Result<(), PredError>,
{
    type Item = I::Item;

    type Output<'a> = C::Output;

    fn collector_test_parts(
        &mut self,
    ) -> CollectorTestParts<
        impl Iterator<Item = Self::Item> + Clone,
        impl Collector<Self::Item, Output = Self::Output<'_>>,
        impl FnMut(Self::Output<'_>, &mut dyn Iterator<Item = Self::Item>) -> Result<(), PredError>,
    > {
        CollectorTestParts {
            iter: (self.iter_factory)(),
            collector: (self.collector_factory)(),
            should_break: (self.should_break_pred)((self.iter_factory)()),
            pred: |output, it| (self.pred)((self.iter_factory)(), output, it),
        }
    }
}

fn test_collector_part(
    tester: &mut (impl CollectorTester + ?Sized),
    collector_fused: bool,
) -> TestCaseResult {
    // `collect()`
    // Introduce scope so that `test_parts` is dropped,
    // or else we get the "mutable more than once" error.
    {
        let mut test_parts = tester.collector_test_parts();
        // Simulate the fact that break_hint is used before looping,
        // which is the intended use case.
        let has_stopped = test_parts.collector.break_hint()
            || test_parts
                .iter
                .try_for_each(|item| test_parts.collector.collect(item))
                .is_break();
        prop_assert_eq!(
            has_stopped,
            test_parts.should_break,
            "`collect()` didn't break correctly"
        );

        if has_stopped && collector_fused {
            for item in test_parts.iter.clone() {
                prop_assert!(
                    test_parts.collector.collect(item).is_break(),
                    "`collect()` isn't actually fused"
                );
            }
        }
        // We may have not considered that the collector is implemeted incorrectly
        // and even if the above test passes, the output of the collector
        // may have been "tainted" by extra items fed.
        // We will catch it also in the below test

        (test_parts.pred)(test_parts.collector.finish(), &mut test_parts.iter)
            .map_err(|e| e.of_method("collect"))?;
    }

    // `collect_many()`
    {
        let mut test_parts = tester.collector_test_parts();
        // We don't call `break_hint()` because it's NOT an intended use case.
        // The user should be able to call it directly without that method.
        let has_stopped = test_parts
            .collector
            .collect_many(&mut test_parts.iter)
            .is_break();
        prop_assert_eq!(
            has_stopped,
            test_parts.should_break,
            "`collect_many()` didn't break correctly"
        );

        if has_stopped && collector_fused {
            prop_assert!(
                test_parts
                    .collector
                    .collect_many(test_parts.iter.clone())
                    .is_break(),
                "`collect_many()` isn't actually fused"
            );
        }

        (test_parts.pred)(test_parts.collector.finish(), &mut test_parts.iter)
            .map_err(|e| e.of_method("collect_many"))?;
    }

    // `collect_then_finish()`
    {
        let mut test_parts = tester.collector_test_parts();
        (test_parts.pred)(
            test_parts
                .collector
                .collect_then_finish(&mut test_parts.iter),
            &mut test_parts.iter,
        )
        .map_err(|e| e.of_method("collect_then_finish"))?;
    }

    Ok(())
}
