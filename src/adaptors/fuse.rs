use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

pub struct Fuse<C> {
    collector: C,
    finished: bool,
}

impl<C> Fuse<C> {
    #[inline]
    pub(crate) fn new(collector: C) -> Self {
        Self {
            collector,
            finished: false,
        }
    }

    #[inline]
    fn collect_impl(&mut self, f: impl FnOnce(&mut C) -> ControlFlow<()>) -> ControlFlow<()> {
        if self.finished {
            ControlFlow::Break(())
        } else if f(&mut self.collector).is_break() {
            self.finished = true;
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl<C: Collector> Collector for Fuse<C> {
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

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.collect_impl(|collector| collector.collect_many(items))
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        if self.finished {
            self.finish()
        } else {
            self.collector.collect_then_finish(items)
        }
    }
}

impl<C: RefCollector> RefCollector for Fuse<C> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.collect_impl(|collector| collector.collect_ref(item))
    }
}
