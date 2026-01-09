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
        impl FnMut(Self::Output<'_>) -> bool,
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
        impl FnMut(Self::Output<'_>) -> bool,
    >;
}

pub struct CollectorTestParts<I, C, P>
where
    I: Iterator,
    C: Collector<Item = I::Item>,
    P: FnMut(C::Output) -> bool,
{
    pub iter: I,
    pub collector: C,
    pub should_break: bool,
    pub output_pred: P,
}

pub trait CollectorTesterExt: CollectorTester {
    #[allow(unused)] // FIXME: delete it when we need it in the future
    fn test_collector(&mut self) -> TestCaseResult {
        test_collector_part(self)?.assert_all_eq()
    }

    fn test_ref_collector(&mut self) -> TestCaseResult
    where
        Self: RefCollectorTester,
    {
        let mut iter_remainders = test_collector_part(self)?;

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
            "`collect()` didn't break correctly"
        );
        prop_assert!(
            (test_parts.output_pred)(test_parts.collector.finish()),
            "`collect_ref()`'s result mismatched"
        );
        iter_remainders.collect_ref = Some(test_parts.iter.count());

        iter_remainders.assert_all_eq()
    }
}

impl<T> CollectorTesterExt for T where T: CollectorTester {}

/// Basic implementation for [`CollectorTester`] for most use case.
/// Opt-out if you test the `collector(_mut)` variant, or the collector and output
/// that may borrow from the tester, or the output is judged differently.
pub struct BasicCollectorTester<ItFac, ClFac, SbPred, OutFac, I, C>
where
    I: Iterator,
    C: Collector<Item = I::Item, Output: PartialEq + Debug>,
    ItFac: FnMut() -> I,
    ClFac: FnMut() -> C,
    SbPred: FnMut(I) -> bool,
    OutFac: FnMut(I) -> C::Output,
{
    pub iter_factory: ItFac,
    pub collector_factory: ClFac,
    pub should_break_pred: SbPred,
    pub output_factory: OutFac,
}

impl<ItFac, ClFac, SbPred, OutFac, I, C> CollectorTester
    for BasicCollectorTester<ItFac, ClFac, SbPred, OutFac, I, C>
where
    I: Iterator,
    C: Collector<Item = I::Item, Output: PartialEq + Debug>,
    ItFac: FnMut() -> I,
    ClFac: FnMut() -> C,
    SbPred: FnMut(I) -> bool,
    OutFac: FnMut(I) -> C::Output,
{
    type Item = I::Item;

    type Output<'a> = C::Output;

    fn collector_test_parts(
        &mut self,
    ) -> CollectorTestParts<
        impl Iterator<Item = Self::Item>,
        impl Collector<Item = Self::Item, Output = Self::Output<'_>>,
        impl FnMut(Self::Output<'_>) -> bool,
    > {
        let expected_output = (self.output_factory)((self.iter_factory)());

        CollectorTestParts {
            iter: (self.iter_factory)(),
            collector: (self.collector_factory)(),
            should_break: (self.should_break_pred)((self.iter_factory)()),
            output_pred: move |output| output == expected_output,
        }
    }
}

impl<ItFac, ClFac, SbPred, OutFac, I, C> RefCollectorTester
    for BasicCollectorTester<ItFac, ClFac, SbPred, OutFac, I, C>
where
    I: Iterator,
    C: RefCollector<Item = I::Item, Output: PartialEq + Debug>,
    ItFac: FnMut() -> I,
    ClFac: FnMut() -> C,
    SbPred: FnMut(I) -> bool,
    OutFac: FnMut(I) -> C::Output,
{
    fn ref_collector_test_parts(
        &mut self,
    ) -> CollectorTestParts<
        impl Iterator<Item = Self::Item>,
        impl RefCollector<Item = Self::Item, Output = Self::Output<'_>>,
        impl FnMut(Self::Output<'_>) -> bool,
    > {
        let expected_output = (self.output_factory)((self.iter_factory)());

        CollectorTestParts {
            iter: (self.iter_factory)(),
            collector: (self.collector_factory)(),
            should_break: (self.should_break_pred)((self.iter_factory)()),
            output_pred: move |output| output == expected_output,
        }
    }
}

fn test_collector_part(
    tester: &mut (impl CollectorTester + ?Sized),
) -> Result<IterRemainders, TestCaseError> {
    let mut iter_remainders = IterRemainders::default();

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
        prop_assert!(
            (test_parts.output_pred)(test_parts.collector.finish()),
            "`collect()`'s result mismatched"
        );
        iter_remainders.collect = test_parts.iter.count();
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
        prop_assert!(
            (test_parts.output_pred)(test_parts.collector.finish()),
            "`collect_many()`'s result mismatched"
        );
        iter_remainders.collect_many = test_parts.iter.count();
    }

    // `collect_then_finish()`
    {
        let mut test_parts = tester.collector_test_parts();
        prop_assert!(
            (test_parts.output_pred)(
                test_parts
                    .collector
                    .collect_then_finish(&mut test_parts.iter)
            ),
            "`collect_then_finish()`'s result mismatched"
        );
        iter_remainders.collect_then_finish = test_parts.iter.count();
    }

    Ok(iter_remainders)
}

#[derive(Debug, Default)]
struct IterRemainders {
    collect: usize,
    collect_many: usize,
    collect_then_finish: usize,
    collect_ref: Option<usize>,
}

impl IterRemainders {
    fn assert_all_eq(&self) -> TestCaseResult {
        let remainders = [self.collect, self.collect_many, self.collect_then_finish]
            .into_iter()
            .chain(self.collect_ref);

        prop_assert!(
            remainders.is_sorted_by(|a, b| a == b),
            "collect methods consume iterator inconsistently. {:?}",
            self,
        );

        Ok(())
    }
}
