///
pub trait AggregateOp {
    type Key;

    type Value;

    type AggregatedValue;

    fn new(&mut self, key: &Self::Key, value: Self::AggregatedValue) -> Self::Value;

    ///
    fn modify(&mut self, entry_value: &mut Self::Value, value: Self::AggregatedValue);
}
