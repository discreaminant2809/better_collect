use std::ops::ControlFlow;

use crate::Collector;

pub struct Max<T> {
    max: Option<T>,
}

impl<T: Ord> Collector for Max<T> {
    type Item = T;

    type Output = Option<T>;

    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        // Because it's `Max`, if `max` is a `None` then it's always smaller than a `Some`.
        // Doesn't work on `Min`, however.
        self.max = self.max.take().max(Some(item));
        ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Output {
        self.max
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        if let Some(max) = items.into_iter().max() {
            self.collect(max)
        } else {
            ControlFlow::Continue(())
        }
    }
}
