use std::ops::ControlFlow;

use crate::{Collector, Fuse, RefCollector};

/// A [`Collector`] that destructures each 2-tuple `(A, B)` item and distributes its fields:
/// `A` goes to the first collector, and `B` goes to the second collector.
///
/// This `struct` is created by [`Collector::unzip()`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Unzip<C1, C2> {
    // `Fuse` is neccessary since either may end earlier.
    // It can ease the implementation.
    collector1: Fuse<C1>,
    collector2: Fuse<C2>,
}

impl<C1, C2> Unzip<C1, C2> {
    pub(crate) fn new(collector1: C1, collector2: C2) -> Self {
        Self {
            collector1: Fuse::new(collector1),
            collector2: Fuse::new(collector2),
        }
    }
}

impl<C1, C2> Collector for Unzip<C1, C2>
where
    C1: Collector,
    C2: Collector,
{
    type Item = (C1::Item, C2::Item);
    type Output = (C1::Output, C2::Output);

    fn collect(&mut self, (item1, item2): Self::Item) -> ControlFlow<()> {
        let res1 = self.collector1.collect(item1);
        let res2 = self.collector2.collect(item2);

        if res1.is_break() && res2.is_break() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn finish(self) -> Self::Output {
        (self.collector1.finish(), self.collector2.finish())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        match items.try_for_each(|(item1, item2)| {
            if self.collector1.collect(item1).is_break() {
                return ControlFlow::Break(Which::First { item2 });
            }

            self.collector2.collect(item2).map_break(|_| Which::Second)
        }) {
            ControlFlow::Continue(_) => ControlFlow::Continue(()),
            ControlFlow::Break(Which::First { item2 }) => self
                .collector2
                .collect_many(Some(item2).into_iter().chain(items.map(|(_, item2)| item2))),
            ControlFlow::Break(Which::Second) => {
                self.collector1.collect_many(items.map(|(item1, _)| item1))
            }
        }
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        let mut items = items.into_iter();

        match items.try_for_each(|(item1, item2)| {
            if self.collector1.collect(item1).is_break() {
                return ControlFlow::Break(Which::First { item2 });
            }

            self.collector2.collect(item2).map_break(|_| Which::Second)
        }) {
            ControlFlow::Continue(_) => self.finish(),
            ControlFlow::Break(Which::First { item2 }) => (
                self.collector1.finish(),
                self.collector2.collect_then_finish(
                    Some(item2).into_iter().chain(items.map(|(_, item2)| item2)),
                ),
            ),
            ControlFlow::Break(Which::Second) => (
                self.collector1
                    .collect_then_finish(items.map(|(item1, _)| item1)),
                self.collector2.finish(),
            ),
        }
    }
}

impl<C1, C2> RefCollector for Unzip<C1, C2>
where
    C1: RefCollector,
    C2: RefCollector,
{
    fn collect_ref(&mut self, (item1, item2): &mut Self::Item) -> ControlFlow<()> {
        let res1 = self.collector1.collect_ref(item1);
        let res2 = self.collector2.collect_ref(item2);

        if res1.is_break() && res2.is_break() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

enum Which<T> {
    First { item2: T },
    Second,
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use proptest::collection::vec as propvec;
    use proptest::prelude::*;

    use crate::prelude::*;

    proptest! {
        #[test]
        fn collect_many(
            vec1 in propvec(any::<i32>(), 0..100),
        ) {
            let results = [iter_way, collect_many_way, collect_then_finish_way]
                .map(|f| f(&vec1));

            prop_assert_eq!(&results[0], &results[1]);
            prop_assert_eq!(&results[0], &results[2]);
            prop_assert_eq!(&results[1], &results[2]);
        }
    }

    fn iter_way(vec1: &[i32]) -> (Vec<i32>, Vec<i32>) {
        get_iter(vec1).unzip()
    }

    fn collect_many_way(vec1: &[i32]) -> (Vec<i32>, Vec<i32>) {
        let mut collector = vec![].into_collector().unzip(vec![]);
        assert!(collector.collect_many(get_iter(vec1)).is_continue());
        collector.finish()
    }

    fn collect_then_finish_way(vec1: &[i32]) -> (Vec<i32>, Vec<i32>) {
        vec![]
            .into_collector()
            .unzip(vec![])
            .collect_then_finish(get_iter(vec1))
    }

    fn get_iter(vec1: &[i32]) -> impl Iterator<Item = (i32, i32)> {
        vec1.iter().copied().map(|num| (num, num))
    }
}
