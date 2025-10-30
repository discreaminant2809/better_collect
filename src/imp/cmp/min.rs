use std::ops::ControlFlow;

use crate::Collector;

#[derive(Debug)]
pub struct Min<T> {
    min: Option<T>,
}

impl<T> Min<T> {
    #[inline]
    pub const fn new() -> Self {
        Min { min: None }
    }
}

impl<T> Default for Min<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Collector<T> for Min<T> {
    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.min = Some(match self.min.take() {
            Some(min) => min.min(item),
            None => item,
        });

        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.min
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        if let Some(min) = items.into_iter().min() {
            self.collect(min)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        items.into_iter().min().min(self.min)
    }
}
