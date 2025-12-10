//! Re-exports commonly used items from this crate.
//!
//! This module is intended to be imported with a wildcard, providing
//! convenient access to the most frequently used traits and types.
//!
//! # Example
//!
//! ```
//! use better_collect::prelude::*;
//! ```

pub use crate::accum_hint::AccumHint;
#[cfg(feature = "alloc")]
pub use crate::string::{ConcatStr, ConcatString};
pub use crate::traits::*;
