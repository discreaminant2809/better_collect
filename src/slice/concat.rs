mod concat_mut;
mod into_concat;

pub use concat_mut::*;
pub use into_concat::*;

/// Converts a container into [`Collector`]s that concatenate slices.
///
/// This trait is currently sealed. It only serves to add methods
/// for types that can hold the concatenation result.
///
/// See its implementors for examples, and see [`ConcatItem`]
/// for possible [`Item`] types.
///
/// [`Collector`]: crate::collector::Collector
/// [`Item`]: crate::collector::Collector::Item
#[allow(private_bounds)]
pub trait Concat: Sized + ConcatSealed {
    /// Creates a [`RefCollector`] that concatenate slices.
    /// The [`Output`] type is the wrapped type.
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`Output`]: crate::collector::Collector::Output
    #[inline]
    fn into_concat<T>(self) -> IntoConcat<Self, T>
    where
        T: ConcatItem<Self>,
    {
        IntoConcat::new(self)
    }

    /// Creates a [`RefCollector`] that concatenate slices into a mutable reference.
    /// The [`Output`] type is the mutable reference of the wrapped type.
    ///
    /// [`RefCollector`]: crate::collector::RefCollector
    /// [`Output`]: crate::collector::Collector::Output
    #[inline]
    fn concat_mut<T>(&mut self) -> ConcatMut<'_, Self, T>
    where
        T: ConcatItem<Self>,
    {
        ConcatMut::new(self)
    }
}

/// Marks a type that can be used as the slice type for the [`Concat`]'s methods,
/// and hence be [`Item`] type for the [`Concat`]'s [`Collector`]s.
///
/// This trait is currently sealed. It only serves to determine
/// which types can be used to concat into which types.
///
/// [`Collector`]: crate::collector::Collector
/// [`Item`]: crate::collector::Collector::Item
#[allow(private_bounds)]
pub trait ConcatItem<OwnedSlice>: Sized + ConcatItemSealed<OwnedSlice> {}

pub(crate) trait ConcatSealed {}

pub(crate) trait ConcatItemSealed<OwnedSlice>: Sized {
    fn push_to(&mut self, owned_slice: &mut OwnedSlice);

    #[inline]
    fn push_into(mut self, owned_slice: &mut OwnedSlice) {
        self.push_to(owned_slice);
    }

    fn bulk_push_into(items: impl IntoIterator<Item = Self>, owned_slice: &mut OwnedSlice) {
        items
            .into_iter()
            .for_each(move |item| item.push_into(owned_slice));
    }
}
