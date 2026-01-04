use super::Collector;

/// Conversion into a [`Collector`].
///
/// By implementing this trait for a type, you define how it will be converted to a collector.
///
/// # Usage in trait bounds
///
/// Using `IntoCollector` in trait bounds allows a function to be generic over both
/// [`Collector`] and `IntoCollector`.
/// This is convenient for users of the function, so when they are using it
/// they do not have to make an extra call to
/// [`IntoCollector::into_collector()`] to obtain an instance of [`Collector`].
pub trait IntoCollector {
    /// The type of the items being collected.
    type Item;

    /// The output of the collector.
    type Output;

    /// Which collector being produced?
    type IntoCollector: Collector<Item = Self::Item, Output = Self::Output>;

    /// Creates a collector from a value.
    fn into_collector(self) -> Self::IntoCollector;
}

impl<C: Collector> IntoCollector for C {
    type Item = C::Item;

    type Output = C::Output;

    type IntoCollector = C;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        self
    }
}
