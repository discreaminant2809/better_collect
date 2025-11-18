//! [`Collector`]s for comparing items.
//!
//! This module provides collectors that determine the maximum or minimum
//! values among the items they collect, using different comparison strategies.
//! They correspond to [`Iterator`]’s comparison-related methods, such as
//! [`Iterator::max()`], [`Iterator::min_by()`], and [`Iterator::max_by_key()`].
//!
//! This module corresponds to [`std::cmp`].
//!
//! [`Collector`]: crate::Collector

mod max;
mod max_by;
mod max_by_key;
mod min;
mod min_by;
mod min_by_key;
mod value_key;
// mod is_sorted;
// mod is_sorted_by;
// mod is_sorted_by_key;

pub use max::*;
pub use max_by::*;
pub use max_by_key::*;
pub use min::*;
pub use min_by::*;
pub use min_by_key::*;
