//! Numeric-related [`Collector`]s.
//!
//! This module provides specialized [`Sum`](crate::Sum) and [`Product`](crate::Product)
//! for numeric types in the standard library.
//!
//! This module corresponds to [`std::num`].
//!
//! [`Collector`]: crate::Collector

mod product;
mod sum;

pub use product::*;
pub use sum::*;
