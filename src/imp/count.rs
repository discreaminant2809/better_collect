use std::ops::ControlFlow;

use crate::{Collector, RefCollector};

#[derive(Debug, Default)]
pub struct Count {
    count: usize,
}

impl Count {
    #[inline]
    pub const fn new() -> Self {
        Count { count: 0 }
    }

    #[inline]
    pub const fn get(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn increment(&mut self) {
        // We don't care about overflow.
        // See: https://doc.rust-lang.org/1.90.0/src/core/iter/traits/iterator.rs.html#219-230
        self.count += 1;
    }
}

impl<T> Collector<T> for Count {
    type Output = usize;

    #[inline]
    fn collect(&mut self, _: T) -> ControlFlow<()> {
        self.increment();
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> usize {
        self.count
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.count += items.into_iter().count();
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.count + items.into_iter().count()
    }
}

impl<T> RefCollector<T> for Count {
    #[inline]
    fn collect_ref(&mut self, _: &mut T) -> ControlFlow<()> {
        self.increment();
        ControlFlow::Continue(())
    }
}
