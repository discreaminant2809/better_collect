//! Numeric-related collectors.
//!
//! This module provides specialized [`Sum`](crate::Sum) and [`Product`](crate::Product)
//! for numeric types in the standard library.

mod product;
mod sum;

pub use product::*;
pub use sum::*;
