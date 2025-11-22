use crate::aggregate::OccupiedEntry;

///
pub trait AggregateOp {
    type Key;

    type Value;

    type AggregatedValue;

    fn initialize(&mut self, entry: &Self::Key, value: Self::AggregatedValue) -> Self::Value;

    ///
    fn modify(
        &mut self,
        entry: &mut impl OccupiedEntry<Key = Self::Key, Value = Self::Value>,
        value: Self::AggregatedValue,
    );
}
