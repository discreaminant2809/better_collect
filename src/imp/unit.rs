//! [`Collector`]s for the unit type `()`.
//!
//! [`Collector`]: crate::Collector

use std::ops::ControlFlow;

use crate::{IntoCollector, RefCollector};

/// A [`Collector`] that always stops accumulating.
/// Its [`Output`](crate::Collector::Output) is `()`.
///
/// This struct is created by `().into_collector()`
/// and `().collector()`.
///
/// [`Collector`]: crate::Collector
pub struct Collector(());

macro_rules! into_collector_impl {
    ($ty:ty) => {
        impl IntoCollector for $ty {
            type Item = ();

            type Output = ();

            type IntoCollector = Collector;

            #[inline]
            fn into_collector(self) -> Self::IntoCollector {
                Collector(())
            }
        }
    };
}

into_collector_impl!(());
into_collector_impl!(&());

impl crate::Collector for Collector {
    type Item = ();
    type Output = ();

    #[inline]
    fn collect(&mut self, _item: Self::Item) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    #[inline]
    fn finish(self) -> Self::Output {}

    #[inline]
    fn break_hint(&self) -> bool {
        true
    }

    /// It won't consume any items in an iterator.
    #[inline]
    fn collect_many(&mut self, _items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    /// It won't consume any items in an iterator.
    #[inline]
    fn collect_then_finish(self, _items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        // Nothing worth doing here
    }
}

impl RefCollector for Collector {
    #[inline]
    fn collect_ref(&mut self, _item: &mut Self::Item) -> ControlFlow<()> {
        ControlFlow::Break(())
    }
}
