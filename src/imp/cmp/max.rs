use std::ops::ControlFlow;

use crate::Collector;

#[derive(Debug)]
pub struct Max<T> {
    max: Option<T>,
}

impl<T> Max<T> {
    #[inline]
    pub const fn new() -> Self {
        Max { max: None }
    }
}

impl<T> Default for Max<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Collector<T> for Max<T> {
    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        // Because it's `Max`, if `max` is a `None` then it's always smaller than a `Some`.
        // Doesn't work on `Min`, however.
        self.max = self.max.take().max(Some(item));
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.max
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        if let Some(max) = items.into_iter().max() {
            self.collect(max)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        items.into_iter().max().max(self.max)
    }
}
