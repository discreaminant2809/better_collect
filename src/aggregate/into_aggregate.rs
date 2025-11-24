use std::ops::ControlFlow;

use crate::{
    Collector,
    aggregate::{AggregateOp, Group, GroupMap, OccupiedGroup, VacantGroup},
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
    M: GroupMap,
    Op: AggregateOp<Key = M::Key, Value = M::Value>,
{
    type Item = (M::Key, Op::Item);

    type Output = M;

    fn collect(&mut self, (key, value): Self::Item) -> ControlFlow<()> {
        match self.map.group(key) {
            Group::Occupied(mut entry) => self.op.modify(entry.value_mut(), value),
            Group::Vacant(entry) => {
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
