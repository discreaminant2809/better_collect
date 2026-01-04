//! Numeric-related [`Collector`]s.
//!
//! This module provides specialized [`Sum`](crate::ops::Sum) and [`Product`](crate::ops::Product)
//! for numeric types in the standard library.
//!
//! This module corresponds to [`std::num`].
//!
//! [`Collector`]: crate::collector::Collector

mod product;
mod sum;

pub use product::*;
pub use sum::*;
