use std::ops::ControlFlow;

use crate::{Collector, Fuse, RefCollector};

#[derive(Debug, Clone)]
pub struct Chain<C1, C2> {
    collector1: Fuse<C1>,
    collector2: C2,
}

impl<C1, C2> Chain<C1, C2> {
    pub(crate) fn new(collector1: C1, collector2: C2) -> Self {
        Self {
            collector1: Fuse::new(collector1),
            collector2,
        }
    }
}

impl<T, C1, C2> Collector<T> for Chain<C1, C2>
where
    C1: Collector<T>,
    C2: Collector<T>,
{
    type Output = (C1::Output, C2::Output);

    fn collect(&mut self, item: T) -> ControlFlow<()> {
        if !self.collector1.finished() {
            self.collector1.collect(item)
        } else {
            self.collector2.collect(item)
        }
    }

    fn finish(self) -> Self::Output {
        (self.collector1.finish(), self.collector2.finish())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        // No need to consult the `fisnished` flag
        if self.collector1.collect_many(items.by_ref()).is_break() {
            self.collector2.collect_many(items)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        let mut items = items.into_iter();

        (
            self.collector1.collect_then_finish(items.by_ref()),
            self.collector2.collect_then_finish(items),
        )
    }
}

impl<E, C1, C2> RefCollector<E> for Chain<C1, C2>
where
    C1: RefCollector<E>,
    C2: RefCollector<E>,
{
    fn collect_ref(&mut self, item: &mut E) -> ControlFlow<()> {
        if !self.collector1.finished() {
            self.collector1.collect_ref(item)
        } else {
            self.collector2.collect_ref(item)
        }
    }
}
