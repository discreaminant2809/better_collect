use std::{marker::PhantomData, ops::ControlFlow};

use crate::Collector;

pub struct Map<C, T, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<C, T, F> Map<C, T, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<T, U, C: Collector<U>, F: FnMut(T) -> U> Collector<T> for Map<C, T, F> {
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.collector.collect((self.f)(item))
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.collector
            .collect_many(items.into_iter().map(&mut self.f))
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.collector
            .collect_then_finish(items.into_iter().map(self.f))
    }
}
