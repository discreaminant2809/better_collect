use std::fmt::Debug;

use proptest::{prelude::*, test_runner::TestCaseResult};

use crate::collector::{Collector, RefCollector};

/// Test helper that returns parts needed for collector proptest.
///
/// # Notes
///
/// The [`Output`] should be reset for every call. May not needed
///  if you can make the output consistent without resetting.
///
/// [`Output`]: CollectorTester::Output
pub trait CollectorTester {
    type Item;

    type Output<'a>;

    #[allow(clippy::type_complexity)] // Can't satisfy it so I suppress it.
    fn collector_test_parts(
        &mut self,
    ) -> CollectorTestParts<
        impl Iterator<Item = Self::Item>,
        impl Collector<Item = Self::Item, Output = Self::Output<'_>>,
        impl FnMut(Self::Output<'_>, &mut dyn Iterator<Item = Self::Item>) -> Result<(), PredError>,
    >;
}

/// Test helper that returns parts needed for ref collector proptest.
///
/// If your tester implements it, its [`CollectorTester`] only needs
/// to forward the call to this.
///
/// The current limitation forces us to have two traits instead of one.
pub trait RefCollectorTester: CollectorTester {
    #[allow(clippy::type_complexity)]
    fn ref_collector_test_parts(
        &mut self,
    ) -> CollectorTestParts<
        impl Iterator<Item = Self::Item>,
        impl RefCollector<Item = Self::Item, Output = Self::Output<'_>>,
        impl FnMut(Self::Output<'_>, &mut dyn Iterator<Item = Self::Item>) -> Result<(), PredError>,
    >;
}

/// Test parts for collector testing.
pub struct CollectorTestParts<I, C, P>
where
    I: Iterator,
    C: Collector<Item = I::Item>,
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
    #[allow(unused)] // FIXME: delete it when we need it in the future
    fn test_collector(&mut self) -> TestCaseResult {
        test_collector_part(self)
    }

    fn test_ref_collector(&mut self) -> TestCaseResult
    where
        Self: RefCollectorTester,
    {
        // `collect_ref()`
        let mut test_parts = self.ref_collector_test_parts();
        // Simulate the fact that break_hint is used before looping,
        // which is the intended use case.
        let has_stopped = test_parts.collector.break_hint()
            || test_parts
                .iter
                .try_for_each(|mut item| test_parts.collector.collect_ref(&mut item))
                .is_break();
        prop_assert_eq!(
            has_stopped,
            test_parts.should_break,
            "`collect_ref()` didn't break correctly"
        );
        (test_parts.pred)(test_parts.collector.finish(), &mut test_parts.iter)
            .map_err(|e| e.of_method("collect_ref"))?;

        Ok(())
    }
}

impl<T> CollectorTesterExt for T where T: CollectorTester {}

/// Basic implementation for [`CollectorTester`] for most use case.
/// Opt-out if you test the `collector(_mut)` variant, or the collector and output
/// that may borrow from the tester, or the output is judged differently.
pub struct BasicCollectorTester<ItFac, ClFac, SbPred, OutFac, ItPred, I, C>
// `where` bound is needed otherwise we get "type annotation needed" for the input iterator.
where
    I: Iterator,
    C: Collector<Item = I::Item, Output: PartialEq>,
    ItFac: FnMut() -> I,
    ClFac: FnMut() -> C,
    SbPred: FnMut(I) -> bool,
    OutFac: FnMut(I) -> C::Output,
    ItPred: FnMut(&mut dyn Iterator<Item = I::Item>) -> bool,
{
    pub iter_factory: ItFac,
    pub collector_factory: ClFac,
    pub should_break_pred: SbPred,
    pub output_factory: OutFac,
    pub iter_pred: ItPred,
}

impl<ItFac, ClFac, SbPred, OutFac, ItPred, I, C> CollectorTester
    for BasicCollectorTester<ItFac, ClFac, SbPred, OutFac, ItPred, I, C>
where
    I: Iterator,
    C: Collector<Item = I::Item, Output: PartialEq>,
    ItFac: FnMut() -> I,
    ClFac: FnMut() -> C,
    SbPred: FnMut(I) -> bool,
    OutFac: FnMut(I) -> C::Output,
    ItPred: FnMut(&mut dyn Iterator<Item = I::Item>) -> bool,
{
    type Item = I::Item;

    type Output<'a> = C::Output;

    fn collector_test_parts(
        &mut self,
    ) -> CollectorTestParts<
        impl Iterator<Item = Self::Item>,
        impl Collector<Item = Self::Item, Output = Self::Output<'_>>,
        impl FnMut(Self::Output<'_>, &mut dyn Iterator<Item = Self::Item>) -> Result<(), PredError>,
    > {
        let expected_output = (self.output_factory)((self.iter_factory)());

        CollectorTestParts {
            iter: (self.iter_factory)(),
            collector: (self.collector_factory)(),
            should_break: (self.should_break_pred)((self.iter_factory)()),
            pred: move |output, iter| {
                if output != expected_output {
                    Err(PredError::IncorrectOutput)
                } else if !(self.iter_pred)(iter) {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
    }
}

impl<ItFac, ClFac, SbPred, OutFac, ItPred, I, C> RefCollectorTester
    for BasicCollectorTester<ItFac, ClFac, SbPred, OutFac, ItPred, I, C>
where
    I: Iterator,
    C: RefCollector<Item = I::Item, Output: PartialEq>,
    ItFac: FnMut() -> I,
    ClFac: FnMut() -> C,
    SbPred: FnMut(I) -> bool,
    OutFac: FnMut(I) -> C::Output,
    ItPred: FnMut(&mut dyn Iterator<Item = I::Item>) -> bool,
{
    fn ref_collector_test_parts(
        &mut self,
    ) -> CollectorTestParts<
        impl Iterator<Item = Self::Item>,
        impl RefCollector<Item = Self::Item, Output = Self::Output<'_>>,
        impl FnMut(Self::Output<'_>, &mut dyn Iterator<Item = Self::Item>) -> Result<(), PredError>,
    > {
        let expected_output = (self.output_factory)((self.iter_factory)());

        CollectorTestParts {
            iter: (self.iter_factory)(),
            collector: (self.collector_factory)(),
            should_break: (self.should_break_pred)((self.iter_factory)()),
            pred: move |output, iter| {
                if output != expected_output {
                    Err(PredError::IncorrectOutput)
                } else if !(self.iter_pred)(iter) {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
    }
}

fn test_collector_part(tester: &mut (impl CollectorTester + ?Sized)) -> TestCaseResult {
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
