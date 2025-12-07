use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

/// A [`Collector`] that stops accumulating after collecting the first `n` items.
///
/// This `struct` is created by [`Collector::take()`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Take<C> {
    collector: C,
    // Unspecified if the underlying collector stops accumulating.
    remaining: usize,
}

impl<C> Take<C> {
    pub(crate) fn new(collector: C, n: usize) -> Self {
        Self {
            collector,
            remaining: n,
        }
    }

    #[inline]
    fn collect_impl(&mut self, f: impl FnOnce(&mut C) -> ControlFlow<()>) -> ControlFlow<()> {
        // Must NOT remove it. The user may construct with `take(0)` and
        // because it hasn't yielded Break, it shouldn't panic!
        if self.remaining == 0 {
            return ControlFlow::Break(());
        }

        self.remaining -= 1;
        let cf = f(&mut self.collector);

        if self.remaining == 0 {
            ControlFlow::Break(())
        } else {
            cf
        }
    }
}

impl<C: Collector> Collector for Take<C> {
    type Item = C::Item;
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.collect_impl(|collector| collector.collect(item))
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     let (lower, upper) = self.collector.size_hint();
    //     (
    //         lower.min(self.remaining),
    //         upper.map(|u| u.min(self.remaining)),
    //     )
    // }

    // fn reserve(&mut self, mut additional_min: usize, mut additional_max: Option<usize>) {
    //     additional_min = additional_min.min(self.remaining);
    //     additional_max = Some(additional_max.map_or(self.remaining, |additional_max| {
    //         additional_max.min(self.remaining)
    //     }));

    //     self.collector.reserve(additional_min, additional_max);
    // }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // FIXED: utilize specialization after it's stabilized.

        let mut items = items.into_iter();
        let (lower_sh, _) = items.size_hint();

        // Implementation note: we trust the iterator's hint.

        // The collector may end early. We risk tracking the state wrong?
        // Worry not. By then, the `remaining` becomes useless
        // and acts as a *soft* fuse.
        if self.remaining <= lower_sh {
            let n = self.remaining;
            self.remaining = 0;
            return self.collector.collect_many(items.take(n));
        }

        self.remaining -= lower_sh;
        self.collector.collect_many(items.by_ref().take(lower_sh))?;

        // We don't know how many left after the lower bound,
        // so we carefully track the state with `inspect`.
        self.collector.collect_many(
            items
                .take(self.remaining)
                // Since the collector may not collect all `remaining` items
                .inspect(|_| self.remaining -= 1),
        )
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        // No need to track the state anymore - we'll be gone!
        self.collector
            .collect_then_finish(items.into_iter().take(self.remaining))
    }
}

impl<C: RefCollector> RefCollector for Take<C> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.collect_impl(|collector| collector.collect_ref(item))
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
            vec1 in propvec(any::<i32>(), ..100),
            vec2 in propvec(any::<i32>(), ..100),
            take_count in ..250_usize,
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

    fn iter_way(vec1: &[i32], vec2: &[i32], take_count: usize) -> Vec<i32> {
        get_iter(vec1, vec2).take(take_count).collect()
    }

    fn new_collector(take_count: usize) -> impl RefCollector<Item = i32, Output = Vec<i32>> {
        vec![].into_collector().take(take_count)
    }

    fn collect_way(vec1: &[i32], vec2: &[i32], take_count: usize) -> Vec<i32> {
        let mut collector = new_collector(take_count);
        let _ = get_iter(vec1, vec2).try_for_each(|item| collector.collect(item));
        collector.finish()
    }

    fn collect_ref_way(vec1: &[i32], vec2: &[i32], take_count: usize) -> Vec<i32> {
        let mut collector = new_collector(take_count);
        let _ = get_iter(vec1, vec2).try_for_each(|mut item| collector.collect_ref(&mut item));
        collector.finish()
    }

    fn collect_many_way(vec1: &[i32], vec2: &[i32], take_count: usize) -> Vec<i32> {
        let mut collector = vec![].into_collector().take(take_count);
        assert!(collector.collect_many(get_iter(vec1, vec2)).is_continue());
        collector.finish()
    }

    fn collect_then_finish_way(vec1: &[i32], vec2: &[i32], take_count: usize) -> Vec<i32> {
        vec![]
            .into_collector()
            .take(take_count)
            .collect_then_finish(get_iter(vec1, vec2))
    }

    fn get_iter(vec1: &[i32], vec2: &[i32]) -> impl Iterator<Item = i32> {
        vec1.iter()
            .copied()
            .chain(vec2.iter().copied().filter(|num| num % 2 != 0))
    }
}
