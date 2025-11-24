use crate::aggregate::AggregateOp;

///
pub trait RefAggregateOp: AggregateOp {
    ///
    fn new_value_ref(&mut self, key: &Self::Key, item: &mut Self::Item) -> Self::Value;

    ///
    fn modify_ref(&mut self, value: &mut Self::Value, item: &mut Self::Item);
}
