use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

#[derive(Debug, Clone)]
pub struct Unzip<C1, C2> {
    collector1: C1,
    collector2: C2,
}

impl<C1, C2> Unzip<C1, C2> {
    pub(crate) fn new(collector1: C1, collector2: C2) -> Self {
        Self {
            collector1,
            collector2,
        }
    }
}

impl<T1, T2, C1, C2> Collector<(T1, T2)> for Unzip<C1, C2>
where
    C1: Collector<T1>,
    C2: Collector<T2>,
{
    type Output = (C1::Output, C2::Output);

    fn collect(&mut self, (item1, item2): (T1, T2)) -> ControlFlow<()> {
        let res1 = self.collector1.collect(item1);
        let res2 = self.collector2.collect(item2);

        res1?;
        res2
    }

    fn finish(self) -> Self::Output {
        (self.collector1.finish(), self.collector2.finish())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = (T1, T2)>) -> ControlFlow<()> {
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

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = (T1, T2)>) -> Self::Output {
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

impl<T1, T2, C1, C2> RefCollector<(T1, T2)> for Unzip<C1, C2>
where
    C1: RefCollector<T1>,
    C2: RefCollector<T2>,
{
    fn collect_ref(&mut self, (item1, item2): &mut (T1, T2)) -> ControlFlow<()> {
        let res1 = self.collector1.collect_ref(item1);
        let res2 = self.collector2.collect_ref(item2);

        res1?;
        res2
    }
}

enum Which<T> {
    First { item2: T },
    Second,
}
