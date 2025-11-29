mod combine;
mod map;

pub use combine::*;
pub use map::*;

/// Defines group's entry manipulation.
pub trait AggregateOp {
    /// The group's key.
    type Key;

    /// The group's value.
    type Value;

    /// What the aggregation operates on?
    type Item;

    /// Creates a new value for a newly created group.
    ///
    /// It must accumulate the provided item right away, not just creating the "default" value.
    fn new_value(&mut self, key: &Self::Key, item: Self::Item) -> Self::Value;

    /// Modifies an existing group's value.
    ///
    /// The current limitations prevent us from providing the key of the group.
    /// That parameter may be added soon.
    fn modify(&mut self, value: &mut Self::Value, item: Self::Item);

    /// Creates an [`AggregateOp`] that that calls a closure on each item before operating on.
    ///
    /// This is used when [`Combine`] expects to operate on `T`,
    /// but you have an aggregate op that operates on `U`. In that case,
    /// you can use `map()` to transform `U` into `T` before passing it along.
    ///
    /// Since it does **not** implement [`RefAggregateOp`], this adaptor should be used
    /// on the **final aggregate op** in [`Combine`], or adapted into a [`RefAggregateOp`]
    /// using the appropriate adaptor.
    /// If you find yourself writing `map().cloned()` or `map().copied()`,
    /// consider using [`map_ref()`](AggregateOp::map_ref) instead, which avoids unnecessary cloning.
    ///
    /// # Examples
    ///
    /// [`RefAggregateOp`]: super::RefAggregateOp
    #[inline]
    fn map<T, F>(self, f: F) -> Map<Self, T, F>
    where
        Self: Sized,
        F: FnMut(T) -> Self::Item,
    {
        Map::new(self, f)
    }
}
