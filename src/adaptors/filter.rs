use crate::{Collector, RefCollector};

use std::{fmt::Debug, ops::ControlFlow};

#[derive(Clone)]
pub struct Filter<C, F> {
    collector: C,
    pred: F,
}

impl<C, F> Filter<C, F> {
    pub(crate) fn new(collector: C, pred: F) -> Self {
        Self { collector, pred }
    }
}

impl<T, C: Collector<T>, F: FnMut(&T) -> bool> Collector<T> for Filter<C, F> {
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        if (self.pred)(&item) {
            self.collector.collect(item)
        } else {
            ControlFlow::Continue(())
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.collector
            .collect_many(items.into_iter().filter(&mut self.pred))
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().filter(self.pred))
    }
}

impl<T, C: RefCollector<T>, F: FnMut(&T) -> bool> RefCollector<T> for Filter<C, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        if (self.pred)(item) {
            self.collector.collect_ref(item)
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl<C: Debug, F> Debug for Filter<C, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Filter")
            .field("collector", &self.collector)
            .finish()
    }
}
