//!

mod adapters;
#[allow(clippy::module_inception)]
mod collector;
mod collector_by_mut;
mod collector_by_ref;
mod into_collector;
mod ref_collector;
mod sink;

pub use adapters::*;
pub use collector::*;
pub use collector_by_mut::*;
pub use collector_by_ref::*;
pub use into_collector::*;
pub use ref_collector::*;
pub use sink::*;
