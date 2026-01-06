//! [`Collector`]s that repeatedly apply common operators in [`std::ops`].
//!
//! This module corresponds to [`std::ops`].
//!
//! [`Collector`]: crate::collector::Collector

mod product;
mod sum;

pub use product::*;
pub use sum::*;
