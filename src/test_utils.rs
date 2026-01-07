use std::fmt::Debug;

use proptest::{prelude::*, test_runner::TestCaseResult};

use crate::prelude::*;

#[allow(unused)] // FIXME: if we use it later, remove this attribute.
pub fn proptest_collector<I, C>(
    mut iter_factory: impl FnMut() -> I,
    collector_factory: impl FnMut() -> C,
    should_break_pred: impl FnOnce(I) -> bool,
    iter_way: impl FnOnce(I) -> C::Output,
) -> TestCaseResult
where
    I: Iterator,
    C: Collector<Item = I::Item, Output: PartialEq + Debug>,
{
    let should_break = should_break_pred(iter_factory());
    let expected_result = iter_way(iter_factory());

    let iter_remainders = proptest_collector_part(
        iter_factory,
        collector_factory,
        should_break,
        &expected_result,
    )?;
    iter_remainders.assert_all_eq()
}

#[allow(unused)]
pub fn proptest_ref_collector<I, C>(
    mut iter_factory: impl FnMut() -> I,
    mut collector_factory: impl FnMut() -> C,
    should_break_pred: impl FnOnce(I) -> bool,
    iter_way: impl FnOnce(I) -> C::Output,
) -> TestCaseResult
where
    I: Iterator,
    C: RefCollector<Item = I::Item, Output: PartialEq + Debug>,
{
    let should_break = should_break_pred(iter_factory());
    let expected_result = iter_way(iter_factory());

    let mut iter_remainders = proptest_collector_part(
        &mut iter_factory,
        &mut collector_factory,
        should_break,
        &expected_result,
    )?;

    // `collect_ref()`
    let mut collector = collector_factory();
    let mut iter = iter_factory();
    // Simulate the fact that break_hint is used before looping,
    // which is the intended use case.
    let has_stopped = collector.break_hint()
        || iter
            .try_for_each(|mut item| collector.collect_ref(&mut item))
            .is_break();
    prop_assert_eq!(
        has_stopped,
        should_break,
        "`collect_ref()` didn't break correctly"
    );
    prop_assert_eq!(
        collector.finish(),
        expected_result,
        "`collect_ref()`'s result mismatched"
    );
    iter_remainders.collect_ref = Some(iter.count());

    iter_remainders.assert_all_eq()
}

fn proptest_collector_part<I, C>(
    mut iter_factory: impl FnMut() -> I,
    mut collector_factory: impl FnMut() -> C,
    should_break: bool,
    expected_result: &C::Output,
) -> Result<IterRemainders, TestCaseError>
where
    I: Iterator,
    C: Collector<Item = I::Item, Output: PartialEq + Debug>,
{
    // `collect()`
    let mut collector = collector_factory();
    let mut iter = iter_factory();
    // Simulate the fact that break_hint is used before looping,
    // which is the intended use case.
    let has_stopped =
        collector.break_hint() || iter.try_for_each(|item| collector.collect(item)).is_break();
    prop_assert_eq!(
        has_stopped,
        should_break,
        "`collect()` didn't break correctly"
    );
    prop_assert_eq!(
        &collector.finish(),
        expected_result,
        "`collect()`'s result mismatched"
    );
    let collect_rem = iter.count();

    // `collect_many()`
    let mut collector = collector_factory();
    let mut iter = iter_factory();
    // We don't call `break_hint()` because it's NOT an intended use case.
    // The user should be able to call it directly without that method.
    let has_stopped = collector.collect_many(&mut iter).is_break();
    prop_assert_eq!(
        has_stopped,
        should_break,
        "`collect_many()` didn't break correctly"
    );
    prop_assert_eq!(
        &collector.finish(),
        expected_result,
        "`collect_many()`'s result mismatched"
    );
    let collect_many_rem = iter.count();

    // `collect_then_finish()`
    let collector = collector_factory();
    let mut iter = iter_factory();
    prop_assert_eq!(
        &collector.collect_then_finish(&mut iter),
        expected_result,
        "`collect_then_finish()`'s result mismatched"
    );
    let collect_then_finish_rem = iter.count();

    Ok(IterRemainders {
        collect: collect_rem,
        collect_many: collect_many_rem,
        collect_then_finish: collect_then_finish_rem,
        collect_ref: None,
    })
}

#[derive(Debug)]
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
