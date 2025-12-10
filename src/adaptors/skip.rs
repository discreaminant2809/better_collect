use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

/// A [`Collector`] that skips the first `n` collected items before it begins
/// accumulating them.
///
/// This `struct` is created by [`Collector::skip()`]. See its documentation for more.
pub struct Skip<C> {
    collector: C,
    remaining: usize,
}

impl<C> Skip<C> {
    pub(crate) fn new(collector: C, n: usize) -> Self {
        Self {
            collector,
            remaining: n,
        }
    }
}

impl<C> Collector for Skip<C>
where
    C: Collector,
{
    type Item = C::Item;

    type Output = C::Output;

    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        if self.remaining > 0 {
            self.remaining -= 1;
            ControlFlow::Continue(())
        } else {
            self.collector.collect(item)
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    #[inline]
    fn has_stopped(&self) -> bool {
        self.remaining == 0 && self.collector.has_stopped()
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // We should ensure that once the iterator ends, we never `next` it again.
        // We don't want to resume it.

        let mut items = items.into_iter();
        // We trust the implementation of `size_hint`.
        let (lower_sh, _) = items.size_hint();

        if self.remaining <= lower_sh {
            let n = std::mem::replace(&mut self.remaining, 0);
            return if drop_n_items(&mut items, n) {
                self.collector.collect_many(items)
            } else {
                ControlFlow::Continue(())
            };
        }

        self.remaining -= lower_sh;

        // Be careful: beyond the lower bound,
        // the iterator may end before skipping all `self.remaining`.
        let mut is_some = drop_n_items(&mut items, lower_sh);
        while is_some && self.remaining > 0 {
            self.remaining -= 1;
            is_some = items.next().is_some();
        }

        if is_some {
            self.collector.collect_many(items)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        let mut items = items.into_iter();

        // `Iterator::skip()` is more strict in TrustedLen implementation,
        // so we manually skip items to preserve the len trustworthiness of the iterator.
        if drop_n_items(&mut items, self.remaining) {
            self.collector.collect_then_finish(items)
        } else {
            self.collector.finish()
        }
    }
}

impl<C> RefCollector for Skip<C>
where
    C: RefCollector,
{
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        if self.remaining > 0 {
            self.remaining -= 1;
            ControlFlow::Continue(())
        } else {
            self.collector.collect_ref(item)
        }
    }
}

// Returns `true` if all n items were dropped (not ended earlier).
// Should consult the returning `bool` to prevent the iterator from "resuming."
fn drop_n_items(items: &mut impl Iterator, n: usize) -> bool {
    if n > 0 {
        items.nth(n - 1).is_some()
    } else {
        true
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use proptest::collection::vec as propvec;
    use proptest::prelude::*;

    use crate::prelude::*;

    proptest! {
        #[test]
        fn collect_many(
            (vec1, vec2, take_count) in prop_oneof![
                9 => (
                    propvec(any::<i32>(), ..100),
                    propvec(any::<i32>(), ..100),
                    ..250_usize,
                ),

                // skip_count == total length case
                1 => (
                    propvec(any::<i32>(), ..100),
                    propvec(any::<i32>(), ..100),
                ).prop_map(|(vec1, vec2)| {
                    let take_count = get_iter(&vec1, &vec2).count();
                    (vec1, vec2, take_count)
                }),
            ]
        ) {
            let fns = [iter_way, collect_way, collect_ref_way, collect_many_way, collect_then_finish_way];
            let mut results = fns
                .into_iter()
                .map(|f| f(&vec1, &vec2, take_count))
                .enumerate();

            let (_, expected) = results.next().unwrap();
            for (i, res) in results{
                prop_assert_eq!(&expected, &res, "{}-th method failed", i);
            }
        }
    }

    fn iter_way(vec1: &[i32], vec2: &[i32], skip_count: usize) -> Vec<i32> {
        get_iter(vec1, vec2).skip(skip_count).collect()
    }

    fn new_collector(skip_count: usize) -> impl RefCollector<Item = i32, Output = Vec<i32>> {
        vec![].into_collector().skip(skip_count)
    }

    fn collect_way(vec1: &[i32], vec2: &[i32], skip_count: usize) -> Vec<i32> {
        let mut collector = new_collector(skip_count);
        let _ = get_iter(vec1, vec2).try_for_each(|item| collector.collect(item));
        collector.finish()
    }

    fn collect_ref_way(vec1: &[i32], vec2: &[i32], skip_count: usize) -> Vec<i32> {
        let mut collector = new_collector(skip_count);
        let _ = get_iter(vec1, vec2).try_for_each(|mut item| collector.collect_ref(&mut item));
        collector.finish()
    }

    fn collect_many_way(vec1: &[i32], vec2: &[i32], skip_count: usize) -> Vec<i32> {
        let mut collector = new_collector(skip_count);
        assert!(collector.collect_many(get_iter(vec1, vec2)).is_continue());
        collector.finish()
    }

    fn collect_then_finish_way(vec1: &[i32], vec2: &[i32], skip_count: usize) -> Vec<i32> {
        new_collector(skip_count).collect_then_finish(get_iter(vec1, vec2))
    }

    fn get_iter(vec1: &[i32], vec2: &[i32]) -> impl Iterator<Item = i32> {
        vec1.iter()
            .copied()
            .chain(vec2.iter().copied().filter(|&num| num > 0))
    }
}
