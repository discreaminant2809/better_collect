use std::{fmt::Debug, marker::PhantomData, mem::forget, ops::ControlFlow};

use crate::collector::Collector;

/// A [`Collector`] that "[forgets](forget)" every item it collects.
///
/// # Examples
///
/// ```no_run
/// use better_collect::{prelude::*, mem::Forgetting};
///
/// std::iter::repeat(vec![0; 100])
///     // Good luck :)
///     .feed_into(Forgetting::new())
/// ```
pub struct Forgetting<T> {
    _marker: PhantomData<T>,
}

impl<T> Forgetting<T> {
    /// Creates a new instance of this collector.
    #[inline]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Collector for Forgetting<T> {
    type Item = T;

    type Output = ();

    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        forget(item);
        ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Output {}

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        items.into_iter().for_each(forget);
        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        items.into_iter().for_each(forget);
    }
}

impl<T> Clone for Forgetting<T> {
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for Forgetting<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forgetting").finish()
    }
}

impl<T> Default for Forgetting<T> {
    fn default() -> Self {
        Self::new()
    }
}
