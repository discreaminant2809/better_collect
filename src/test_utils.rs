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

    proptest_collector_part(
        iter_factory,
        collector_factory,
        should_break,
        &expected_result,
    )
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

    proptest_collector_part(
        &mut iter_factory,
        &mut collector_factory,
        should_break,
        &expected_result,
    )?;

    // `collect_ref()`
    let mut collector = collector_factory();
    // Simulate the fact that break_hint is used before looping,
    // which is the intended use case.
    let has_stopped = collector.break_hint()
        || iter_factory()
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

    Ok(())
}

fn proptest_collector_part<I, C>(
    mut iter_factory: impl FnMut() -> I,
    mut collector_factory: impl FnMut() -> C,
    should_break: bool,
    expected_result: &C::Output,
) -> TestCaseResult
where
    I: Iterator,
    C: Collector<Item = I::Item, Output: PartialEq + Debug>,
{
    // `collect()`
    let mut collector = collector_factory();
    // Simulate the fact that break_hint is used before looping,
    // which is the intended use case.
    let has_stopped = collector.break_hint()
        || iter_factory()
            .try_for_each(|item| collector.collect(item))
            .is_break();
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

    // `collect_many()`
    let mut collector = collector_factory();
    // We don't call `break_hint()` because it's NOT an intended use case.
    // The user should be able to call it directly without that method.
    let has_stopped = collector.collect_many(iter_factory()).is_break();
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

    // `collect_then_finish()`
    let collector = collector_factory();
    prop_assert_eq!(
        &collector.collect_then_finish(iter_factory()),
        expected_result,
        "`collect_then_finish()`'s result mismatched"
    );

    Ok(())
}
