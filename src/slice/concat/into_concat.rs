use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::collector::{Collector, RefCollector};

use super::ConcatItem;

/// A [`RefCollector`] that concatenates items.
///
/// This `struct` is created by [`Concat::into_concat()`]. See its documentation for more.
///
/// [`Concat::into_concat()`]: super::Concat::into_concat
pub struct IntoConcat<S, T> {
    owned_slice: S,
    _marker: PhantomData<fn(T, &mut T)>,
}

impl<S, T> IntoConcat<S, T> {
    pub(super) fn new(owned_slice: S) -> Self {
        Self {
            owned_slice,
            _marker: PhantomData,
        }
    }
}

impl<S, T> Collector for IntoConcat<S, T>
where
    T: ConcatItem<S>,
{
    type Item = T;

    type Output = S;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        item.push_into(&mut self.owned_slice);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.owned_slice
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        T::bulk_push_into(items, &mut self.owned_slice);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        T::bulk_push_into(items, &mut self.owned_slice);
        self.owned_slice
    }
}

impl<S, T> RefCollector for IntoConcat<S, T>
where
    T: ConcatItem<S>,
{
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        item.push_to(&mut self.owned_slice);
        ControlFlow::Continue(())
    }
}

impl<S, T> Clone for IntoConcat<S, T>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            owned_slice: self.owned_slice.clone(),
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.owned_slice.clone_from(&source.owned_slice);
    }
}

impl<S, T> Debug for IntoConcat<S, T>
where
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntoConcat")
            .field("owned_slice", &self.owned_slice)
            .finish()
    }
}

impl<S, T> Default for IntoConcat<S, T>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            owned_slice: Default::default(),
            _marker: PhantomData,
        }
    }
}
