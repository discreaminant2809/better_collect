use std::{iter, ops::ControlFlow};

use crate::collector::{Collector, CollectorBase};

use super::Fuse;

///
#[derive(Debug, Clone)]
pub struct CombineRef<C1, C2> {
    collector1: Fuse<C1>,
    collector2: Fuse<C2>,
}

impl<C1, C2> CombineRef<C1, C2>
where
    C1: CollectorBase,
    C2: CollectorBase,
{
    pub(in crate::collector) fn new(collector1: C1, collector2: C2) -> Self {
        Self {
            collector1: Fuse::new(collector1),
            collector2: Fuse::new(collector2),
        }
    }
}

impl<C1, C2> CollectorBase for CombineRef<C1, C2>
where
    C1: CollectorBase,
    C2: CollectorBase,
{
    type Output = (C1::Output, C2::Output);

    #[inline]
    fn finish(self) -> Self::Output {
        (self.collector1.finish(), self.collector2.finish())
    }

    #[inline]
    fn break_hint(&self) -> ControlFlow<()> {
        // We're sure that whether this collector has finished or not is
        // entirely based on the 2nd collector.
        // Also, by this method being called it is assumed that
        // this collector has not finished, which mean the 2nd collector
        // has not finished, which means it's always sound to call here.
        //
        // Since the 1st collector is fused, we won't cause any unsoundness
        // by repeatedly calling it.
        if self.collector1.break_hint().is_break() && self.collector2.break_hint().is_break() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl<'i, T, C1, C2> Collector<&'i mut T> for CombineRef<C1, C2>
where
    C1: for<'a> Collector<&'a mut T>,
    C2: Collector<&'i mut T>,
{
    fn collect(&mut self, item: &'i mut T) -> ControlFlow<()> {
        match (self.collector1.collect(item), self.collector2.collect(item)) {
            (ControlFlow::Break(_), ControlFlow::Break(_)) => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = &'i mut T>) -> ControlFlow<()> {
        self.break_hint()?;

        let mut items = items.into_iter();

        match items.try_for_each(|item| {
            if self.collector1.collect(item).is_break() {
                ControlFlow::Break(Which::First(item))
            } else {
                self.collector2.collect(item).map_break(|_| Which::Second)
            }
        }) {
            ControlFlow::Break(Which::First(item)) => {
                self.collector2.collect_many(iter::once(item).chain(items))
            }
            ControlFlow::Break(Which::Second) => self.collector1.collect_many(items),
            ControlFlow::Continue(_) => ControlFlow::Continue(()),
        }
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = &'i mut T>) -> Self::Output {
        if self.break_hint().is_break() {
            return self.finish();
        }

        let mut items = items.into_iter();

        match items.try_for_each(|item| {
            if self.collector1.collect(item).is_break() {
                ControlFlow::Break(Which::First(item))
            } else {
                self.collector2.collect(item).map_break(|_| Which::Second)
            }
        }) {
            ControlFlow::Break(Which::First(item)) => (
                self.collector1.finish(),
                self.collector2
                    .collect_then_finish(iter::once(item).chain(items)),
            ),
            ControlFlow::Break(Which::Second) => (
                self.collector1.collect_then_finish(items),
                self.collector2.finish(),
            ),
            ControlFlow::Continue(_) => self.finish(),
        }
    }
}

enum Which<T> {
    First(T),
    Second,
}
