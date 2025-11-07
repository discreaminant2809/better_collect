use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_ref_collector};

/// A [`RefCollector`] that counts the number of items it collects.
///
/// This collector corresponds to [`Iterator::count()`].
///
/// # Overflow Behavior
///
/// This collector does no guarding against overflows, so feeding it
/// more than [`usize::MAX`] items either produces the wrong result or panics.
/// If overflow checks are enabled, a panic is guaranteed.
/// This is similar to [`Iterator::count()`].
///
/// # Examples
///
/// ```
/// use better_collect::{Collector, Count};
///
/// let mut collector = Count::new();
///
/// assert!(collector.collect(3).is_continue());
/// assert!(collector.collect(7).is_continue());
/// assert!(collector.collect(0).is_continue());
/// assert!(collector.collect(-1).is_continue());
///
/// assert_eq!(collector.finish(), 4);
/// ```
pub struct Count<T> {
    count: usize,
    _marker: PhantomData<fn(T)>,
}

impl<T> Count<T> {
    /// Creates a new instance of this collector with an initial count of 0.
    #[inline]
    pub const fn new() -> Self {
        assert_ref_collector(Count {
            count: 0,
            _marker: PhantomData,
        })
    }

    /// Returns the current count.
    #[inline]
    pub const fn get(&self) -> usize {
        self.count
    }

    #[inline]
    fn increment(&mut self) {
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

impl<T> Collector for &mut Count<T> {
    type Item = T;

    type Output = usize;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        Count::collect(self, item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.get()
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        Count::collect_many(self, items)
    }
}

impl<T> RefCollector for &mut Count<T> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        Count::collect_ref(self, item)
    }
}

impl<T> Debug for Count<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Count").field("count", &self.count).finish()
    }
}

impl<T> Clone for Count<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            count: self.count,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for Count<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
