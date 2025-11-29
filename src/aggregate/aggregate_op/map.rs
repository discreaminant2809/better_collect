use core::marker::PhantomData;

use crate::aggregate::AggregateOp;

/// Creates an [`AggregateOp`] that that calls a closure on each item before operating on.
///
/// This `struct` is created by [`AggregateOp::map()`]. See its documentation for more.
pub struct Map<Op, T, F> {
    op: Op,
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<Op, T, F> Map<Op, T, F> {
    pub(super) fn new(op: Op, f: F) -> Self {
        Self {
            op,
            f,
            _marker: PhantomData,
        }
    }
}

impl<Op: AggregateOp, T, F> AggregateOp for Map<Op, T, F>
where
    F: FnMut(T) -> Op::Item,
{
    type Key = Op::Key;

    type Value = Op::Value;

    type Item = T;

    #[inline]
    fn new_value(&mut self, key: &Self::Key, item: Self::Item) -> Self::Value {
        self.op.new_value(key, (self.f)(item))
    }

    #[inline]
    fn modify(&mut self, value: &mut Self::Value, item: Self::Item) {
        self.op.modify(value, (self.f)(item));
    }
}
