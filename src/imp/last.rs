use std::ops::ControlFlow;

use crate::Collector;

#[derive(Debug)]
pub struct Last<T> {
    value: Option<T>,
}

impl<T> Last<T> {
    #[inline]
    pub const fn new() -> Self {
        Last { value: None }
    }
}

impl<T> Collector for Last<T> {
    type Item = T;
    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        self.value = Some(item);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.value
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // We need a bit complication here since we may risk assigning `None` to `self.value` being `Some`.
        match (&mut self.value, items.into_iter().last()) {
            (Some(value), Some(last)) => *value = last,
            // DO NOT update here. `items` don't have a value to "inherit" the last spot.
            (Some(_), None) => {}
            (None, last) => self.value = last,
        }

        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        items.into_iter().last().or(self.value)
    }
}

impl<T> Default for Last<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
