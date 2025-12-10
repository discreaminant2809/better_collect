use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::Collector;

/// A [`Collector`] with a custom collection logic.
///
/// This `struct` is created by [`Collector::unbatching()`]. See its documentation for more.
pub struct Unbatching<C, T, F> {
    collector: C,
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<C, T, F> Unbatching<C, T, F> {
    pub(crate) fn new(collector: C, f: F) -> Self {
        Self {
            collector,
            f,
            _marker: PhantomData,
        }
    }
}

impl<C, T, F> Collector for Unbatching<C, T, F>
where
    C: Collector,
    F: FnMut(&mut C, T) -> ControlFlow<()>,
{
    type Item = T;

    type Output = C::Output;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.collector, item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    // Can't meaningfully override the `finished`.
    // Since the caller may do some other works than accumulating.

    // Can't meaningfully override `collect_many` and `collect_then_finish`.
}

impl<C: Clone, T, F: Clone> Clone for Unbatching<C, T, F> {
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

impl<C: Debug, T, F> Debug for Unbatching<C, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Unbatching")
            .field("collector", &self.collector)
            .finish()
    }
}
