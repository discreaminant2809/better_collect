use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::collector::{Collector, RefCollector};

use super::ConcatItem;

/// A [`RefCollector`] that concatenate slices.
///
/// This `struct` is created by [`Concat::concat_mut()`]. See its documentation for more.
///
/// [`Concat::concat_mut()`]: super::Concat::concat_mut
pub struct ConcatMut<'a, S, T> {
    owned_slice: &'a mut S,
    _marker: PhantomData<fn(T, &mut T)>,
}

impl<'a, S, T> ConcatMut<'a, S, T> {
    pub(super) fn new(owned_slice: &'a mut S) -> Self {
        Self {
            owned_slice,
            _marker: PhantomData,
        }
    }
}

impl<'a, S, T> Collector for ConcatMut<'a, S, T>
where
    T: ConcatItem<S>,
{
    type Item = T;

    type Output = &'a mut S;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        item.push_into(self.owned_slice);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.owned_slice
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        T::bulk_push_into(items, self.owned_slice);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        T::bulk_push_into(items, self.owned_slice);
        self.owned_slice
    }
}

impl<S, T> RefCollector for ConcatMut<'_, S, T>
where
    T: ConcatItem<S>,
{
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        item.push_to(self.owned_slice);
        ControlFlow::Continue(())
    }
}

impl<S, T> Debug for ConcatMut<'_, S, T>
where
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcatMut")
            .field("owned_slice", &self.owned_slice)
            .finish()
    }
}
