use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

/// A [`RefCollector`] that maps a mutable reference to an item
/// into another mutable reference.
///
/// This `struct` is created by [`RefCollector::funnel()`]. See its documentation for more.
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

impl<T, C, F> Collector for Funnel<C, T, F>
where
    C: RefCollector,
    F: FnMut(&mut T) -> &mut C::Item,
{
    type Item = T;
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
    // Anyway this is fine. This collector is not supposed to be used at the last of the `then` chain.
}

impl<T, C, F> RefCollector for Funnel<C, T, F>
where
    C: RefCollector,
    F: FnMut(&mut T) -> &mut C::Item,
{
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
