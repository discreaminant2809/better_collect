mod combine;

pub use combine::*;

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
}
