use std::{marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

pub struct Count<T> {
    count: usize,
    // We need a generic because we can't leave `Collector::Item` unconstrained.
    _marker: PhantomData<T>,
}

impl<T> Count<T> {
    #[inline]
    pub const fn new() -> Self {
        Count {
            count: 0,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn increment(&mut self) {
        // We don't care about overflow.
        // See: https://doc.rust-lang.org/1.90.0/src/core/iter/traits/iterator.rs.html#219-230
        self.count += 1;
    }
}

impl<T> Collector for Count<T> {
    type Item = T;

    type Output = usize;

    #[inline]
    fn collect(&mut self, _: Self::Item) -> ControlFlow<()> {
        self.increment();
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> usize {
        self.count
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.count += items.into_iter().count();
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.count + items.into_iter().count()
    }
}

impl<T> RefCollector for Count<T> {
    #[inline]
    fn collect_ref(&mut self, _: &mut Self::Item) -> ControlFlow<()> {
        self.increment();
        ControlFlow::Continue(())
    }
}

impl<T> Default for Count<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for Count<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Count").field("count", &self.count).finish()
    }
}
