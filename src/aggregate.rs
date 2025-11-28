//! Module containing items for aggregation.

mod aggregate_op;
mod group;
mod group_map;
mod imp;
mod into_aggregate;
mod ref_aggregate_op;

pub use aggregate_op::*;
pub use group::*;
pub use group_map::*;
pub use imp::*;
pub use into_aggregate::*;
pub use ref_aggregate_op::*;

#[inline(always)]
const fn assert_op<Op: AggregateOp>(op: Op) -> Op {
    op
}

#[inline(always)]
const fn assert_ref_op<Op: RefAggregateOp>(op: Op) -> Op {
    op
}
