use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

/// A [`RefCollector`] with a custom collection logic.
///
/// This `struct` is created by [`Collector::unbatching_ref()`]. See its documentation for more.
pub struct UnbatchingRef<C, T, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(&mut T)>,
}

impl<C, T, F> UnbatchingRef<C, T, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<C, T, F> Collector for UnbatchingRef<C, T, F>
where
    C: Collector,
    F: FnMut(&mut C, &mut T) -> ControlFlow<()>,
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

    // Can't meaningfully override `has_stopped()`,
    // since the caller may do some other works than accumulating
    // to the underlying collector.
    // Even if the underlying collector has stopped since creation,
    // the closure may actually return `Continue(())`,
    // rendering the signal incorrect.

    // Can't meaningfully override `collect_many` and `collect_then_finish`.
}

impl<C, T, F> RefCollector for UnbatchingRef<C, T, F>
where
    C: Collector,
    F: FnMut(&mut C, &mut T) -> ControlFlow<()>,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.collector, item)
    }
}

impl<C: Clone, T, F: Clone> Clone for UnbatchingRef<C, T, F> {
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

impl<C: Debug, T, F> Debug for UnbatchingRef<C, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnbatchingRef")
            .field("collector", &self.collector)
            .finish()
    }
}
