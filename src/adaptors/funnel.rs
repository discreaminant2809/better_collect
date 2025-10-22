use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

pub struct Funnel<C, E, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(&mut E)>,
}

impl<C, E, F> Funnel<C, E, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<E, C: RefCollector, F: FnMut(&mut E) -> &mut C::Item> Collector for Funnel<C, E, F> {
    type Item = E;

    type Output = C::Output;

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    // Not doable...
    // fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
    //     self.collector
    //         .collect_many(items.into_iter().map(|mut item| (self.f)(&mut item))) // ...due to this.
    // }
}

impl<E, C: RefCollector, F: FnMut(&mut E) -> &mut C::Item> RefCollector for Funnel<C, E, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.collector.collect_ref((self.f)(item))
    }
}
