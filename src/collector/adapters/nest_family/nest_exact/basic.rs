use std::ops::ControlFlow;

use crate::collector::{Collector, RefCollector};

use super::{super::strategy::CloneStrategy, with_strategy::WithStrategy};

/// A [`Collector`] that collects all outputs produced by an inner collector.
///
/// This `struct` is created by [`Collector::nest_exact()`]. See its documentation for more.
// Needed because the "Available on crate feature" does not show up on doc.rs
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub struct NestExact<CO, CI>(WithStrategy<CO, CloneStrategy<CI>>)
where
    CI: Collector + Clone;

impl<CO, CI> NestExact<CO, CI>
where
    CI: Collector + Clone,
{
    pub(in crate::collector) fn new(outer: CO, inner: CI) -> Self {
        Self(WithStrategy::new(outer, CloneStrategy::new(inner)))
    }
}

impl<CO, CI> Collector for NestExact<CO, CI>
where
    CO: Collector<Item = CI::Output>,
    CI: Collector + Clone,
{
    type Item = CI::Item;

    type Output = CO::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.0.collect(item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0.finish()
    }

    #[inline]
    fn break_hint(&self) -> bool {
        self.0.break_hint()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.0.collect_many(items)
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.0.collect_then_finish(items)
    }
}

impl<CO, CI> RefCollector for NestExact<CO, CI>
where
    CO: Collector<Item = CI::Output>,
    CI: RefCollector + Clone,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.0.collect_ref(item)
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use proptest::collection::vec as propvec;
    use proptest::prelude::*;

    use crate::prelude::*;

    proptest! {
        #[test]
        fn all_collect_methods(
            nums in propvec(any::<i32>(), ..100),
            row in ..25_usize,
            column in 1..25_usize,
        ) {
            let fns = [iter_way, collect_way, collect_ref_way, collect_many_way, collect_then_finish_way];
            let mut results = fns
                .into_iter()
                .map(|f| f(&nums, row, column))
                .enumerate();

            let (_, expected) = results.next().unwrap();
            for (i, res) in results{
                prop_assert_eq!(&expected, &res, "{}-th method failed", i);
            }
        }
    }

    fn iter_way(nums: &[i32], row: usize, column: usize) -> Vec<Vec<i32>> {
        nums.chunks_exact(column).take(row).map(Vec::from).collect()
    }

    fn collect_way(nums: &[i32], row: usize, column: usize) -> Vec<Vec<i32>> {
        let should_break = should_break(nums, row, column);

        let mut collector = get_collector(row, column);

        // Simulate the fact that break_hint is used before looping,
        // which is the intended use case.
        let has_stopped = collector.break_hint()
            || get_iter(nums)
                .try_for_each(|item| collector.collect(item))
                .is_break();

        assert_eq!(has_stopped, should_break);

        collector.finish()
    }

    fn collect_ref_way(nums: &[i32], row: usize, column: usize) -> Vec<Vec<i32>> {
        let should_break = should_break(nums, row, column);

        let mut collector = get_collector(row, column);

        // Simulate the fact that break_hint is used before looping,
        // which is the intended use case.
        let has_stopped = collector.break_hint()
            || get_iter(nums)
                .try_for_each(|mut item| collector.collect_ref(&mut item))
                .is_break();

        assert_eq!(has_stopped, should_break);

        collector.finish()
    }

    fn collect_many_way(nums: &[i32], row: usize, column: usize) -> Vec<Vec<i32>> {
        let should_break = should_break(nums, row, column);

        let mut collector = get_collector(row, column);
        assert_eq!(
            collector.collect_many(get_iter(nums)).is_break(),
            should_break
        );
        collector.finish()
    }

    fn collect_then_finish_way(nums: &[i32], row: usize, column: usize) -> Vec<Vec<i32>> {
        get_collector(row, column).collect_then_finish(get_iter(nums))
    }

    fn get_iter(nums: &[i32]) -> impl Iterator<Item = i32> {
        nums.iter().copied()
    }

    fn get_collector(
        row: usize,
        column: usize,
    ) -> impl RefCollector<Item = i32, Output = Vec<Vec<i32>>> {
        vec![]
            .into_collector()
            .take(row)
            .nest_exact(vec![].into_collector().take(column))
    }

    fn should_break(nums: &[i32], row: usize, column: usize) -> bool {
        nums.len() >= row * column
    }
}
