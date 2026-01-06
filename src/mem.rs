//! [`Collector`]s that deal with memory.
//!
//! This module corresponds to [`std::mem`].
//!
//! [`Collector`]: crate::collector::Collector

mod forgetting;

pub use forgetting::*;
