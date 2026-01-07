use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::collector::{Collector, RefCollector};

/// A [`RefCollector`] that collects items... but no one knows where they go.
///
/// All we know is that it relentlessly consumes them, never to be seen again.
///
/// This collector is the counterpart of [`std::iter::empty()`], just like
/// [`std::io::sink()`] and [`std::io::empty()`].
///
/// # Examples
///
/// It collected the example. Nothing to show.
pub struct Sink<T> {
    _marker: PhantomData<T>,
}

impl<T> Sink<T> {
    /// Creates a new instance of this collector.
    #[inline]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Collector for Sink<T> {
    type Item = T;

    type Output = ();

    #[inline]
    fn collect(&mut self, _item: Self::Item) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {}

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        items.into_iter().for_each(drop);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        items.into_iter().for_each(drop);
    }
}

impl<T> RefCollector for Sink<T> {
    #[inline]
    fn collect_ref(&mut self, _item: &mut Self::Item) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

impl<T> Clone for Sink<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for Sink<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sink").finish()
    }
}

impl<T> Default for Sink<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

    use crate::test_utils::proptest_ref_collector;

    use super::*;

    proptest! {
        #[test]
        fn all_collect_methods(
            count in ..5_usize,
        ) {
            all_collect_methods_impl(count)?;
        }
    }

    fn all_collect_methods_impl(count: usize) -> TestCaseResult {
        proptest_ref_collector(
            || std::iter::repeat_n(0, count),
            Sink::new,
            |_| false,
            |_| {},
        )
    }
}
