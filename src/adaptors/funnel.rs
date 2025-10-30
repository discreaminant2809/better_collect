use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

pub struct Funnel<C, T, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(&mut T)>,
}

impl<C, T, F> Funnel<C, T, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<T, U, C: RefCollector<U>, F: FnMut(&mut T) -> &mut U> Collector<T> for Funnel<C, T, F> {
    type Output = C::Output;

    #[inline]
    fn collect(&mut self, mut item: T) -> ControlFlow<()> {
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

impl<T, U, C: RefCollector<U>, F: FnMut(&mut T) -> &mut U> RefCollector<T> for Funnel<C, T, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        self.collector.collect_ref((self.f)(item))
    }
}

impl<C: Clone, T, F: Clone> Clone for Funnel<C, T, F> {
    fn clone(&self) -> Self {
        Self {
            collector: self.collector.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.collector.clone_from(&source.collector);
        self.f.clone_from(&source.f);
    }
}

impl<C: Debug, T, F> Debug for Funnel<C, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Funnel")
            .field("collector", &self.collector)
            .finish()
    }
}
