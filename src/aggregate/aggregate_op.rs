mod tee;

pub use tee::*;

///
pub trait AggregateOp {
    type Key;

    type Value;

    type Item;

    fn new_value(&mut self, key: &Self::Key, item: Self::Item) -> Self::Value;

    ///
    fn modify(&mut self, value: &mut Self::Value, item: Self::Item);
}

#[inline(always)]
fn assert_op<Op: AggregateOp>(op: Op) -> Op {
    op
}
