use core::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector};

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

macro_rules! impl_collector {
    ($ty:ty) => {
        impl<T> Collector for $ty {
            type Item = T;

            type Output = ();

            #[inline]
            fn collect(&mut self, _item: Self::Item) -> ControlFlow<()> {
                ControlFlow::Continue(())
            }

            #[inline]
            fn finish(self) -> Self::Output {
                // Nothing worth implementing.
            }

            #[inline]
            fn collect_many(
                &mut self,
                items: impl IntoIterator<Item = Self::Item>,
            ) -> ControlFlow<()> {
                items.into_iter().for_each(drop);
                ControlFlow::Continue(())
            }

            #[inline]
            fn collect_then_finish(
                self,
                items: impl IntoIterator<Item = Self::Item>,
            ) -> Self::Output {
                items.into_iter().for_each(drop);
            }
        }

        impl<T> RefCollector for $ty {
            #[inline]
            fn collect_ref(&mut self, _item: &mut Self::Item) -> ControlFlow<()> {
                ControlFlow::Continue(())
            }
        }
    };
}

impl_collector!(Sink<T>);
impl_collector!(&Sink<T>);
impl_collector!(&mut Sink<T>);

impl<T> Clone for Sink<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for Sink<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Sink").finish()
    }
}

impl<T> Default for Sink<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
