use std::{fmt::Debug, mem::forget, ops::ControlFlow};

use crate::collector::{Collector, CollectorBase};

/// A [`Collector`] that "[forgets](forget)" every item it collects.
///
/// # Examples
///
/// ```no_run
/// use better_collect::{prelude::*, mem::Forgetting};
///
/// std::iter::repeat(vec![0; 100])
///     // Good luck :)
///     .feed_into(Forgetting)
/// ```
#[derive(Debug, Clone, Default)]
pub struct Forgetting;

impl CollectorBase for Forgetting {
    type Output = ();

    fn finish(self) -> Self::Output {}
}

impl<T> Collector<T> for Forgetting {
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        forget(item);
        ControlFlow::Continue(())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        items.into_iter().for_each(forget);
        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        items.into_iter().for_each(forget);
    }
}
