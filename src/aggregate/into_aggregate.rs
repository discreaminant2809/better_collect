use std::ops::ControlFlow;

use crate::{
    Collector,
    aggregate::{AggregateOp, Entry, Map, OccupiedEntry, VacantEntry},
};

pub struct IntoAggregate<M, Op> {
    map: M,
    op: Op,
}

impl<M, Op> IntoAggregate<M, Op> {
    pub(super) fn new(map: M, op: Op) -> Self {
        Self { map, op }
    }
}

impl<M, Op> Collector for IntoAggregate<M, Op>
where
    M: Map,
    Op: AggregateOp<Key = M::Key, Value = M::Value>,
{
    type Item = (M::Key, Op::Item);

    type Output = M;

    fn collect(&mut self, (key, value): Self::Item) -> ControlFlow<()> {
        match self.map.entry(key) {
            Entry::Occupied(mut entry) => self.op.modify(entry.value_mut(), value),
            Entry::Vacant(entry) => {
                let value = self.op.new_value(entry.key(), value);
                entry.insert(value);
            }
        }

        ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Output {
        self.map
    }
}
