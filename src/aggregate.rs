//!

mod aggregate_op;
mod entry;
mod into_aggregate;
mod map;
mod ref_aggregate_op;

pub use aggregate_op::*;
pub use entry::*;
pub use map::*;
pub use ref_aggregate_op::*;

#[inline(always)]
pub(crate) fn assert_op<Op: AggregateOp>(op: Op) -> Op {
    op
}
