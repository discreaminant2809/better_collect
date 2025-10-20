use core::ops::ControlFlow;

use crate::RefCollector;

pub(super) struct FuseByRef<C: RefCollector> {
    collector: C,
    finished: bool,
}

impl<C: RefCollector> FuseByRef<C> {
    #[inline]
    pub(super) fn new(collector: C) -> Self {
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

impl<C: RefCollector> RefCollector for FuseByRef<C> {
    type Item = C::Item;

    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: &mut Self::Item) -> core::ops::ControlFlow<()> {
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
}
