//! Module contains traits and `struct`s for collectors.

mod adapters;
#[allow(clippy::module_inception)]
mod collector;
mod collector_base;
mod collector_by_mut;
mod collector_by_ref;
mod into_collector;
mod sink;

pub use adapters::*;
pub use collector::*;
pub use collector_base::*;
pub use collector_by_mut::*;
pub use collector_by_ref::*;
pub use into_collector::*;
pub use sink::*;

#[inline(always)]
pub(crate) const fn assert_collector_base<C>(collector: C) -> C
where
    C: CollectorBase,
{
    collector
}

#[inline(always)]
pub(crate) const fn assert_collector<C, T>(collector: C) -> C
where
    C: Collector<T>,
{
    collector
}
