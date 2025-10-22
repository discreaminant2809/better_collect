use std::ops::ControlFlow;

use crate::Collector;

pub struct Max<T> {
    max: Option<T>,
}

impl<T: Ord> Collector for Max<T> {
    type Item = T;

    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        // Because it's `Max`, if `max` is a `None` then it's always smaller than a `Some`.
        // Doesn't work on `Min`, however.
        self.max = self.max.take().max(Some(item));
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.max
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        if let Some(max) = items.into_iter().max() {
            self.collect(max)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        items.into_iter().chain(self.max).max()
    }
}
