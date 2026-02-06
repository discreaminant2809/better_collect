//! Module contains traits and `struct`s for collectors.
//!
//! # Unspecified behaviors
//!
//! Unless stated otherwise by the collector’s implementation, after any of
//! [`Collector::collect()`], [`Collector::collect_many()`], or
//! [`CollectorBase::break_hint()`] have returned [`Break(())`] once,
//! behaviors of subsequent calls to any method other than
//! [`finish()`](CollectorBase::finish) are unspecified.
//! They may panic, overflow, or even resume accumulation
//! (similar to how [`Iterator::next()`] might yield again after returning [`None`]).
//! Callers should generally call [`finish()`](CollectorBase::finish) once a collector
//! has signaled a stop.
//! If this invariant cannot be upheld, wrap it with [`fuse()`](CollectorBase::fuse).
//!
//! This looseness allows for optimizations (for example, omitting an internal "stopped” flag).
//!
//! Although the behavior is unspecified, none of the aforementioned methods are `unsafe`.
//! Implementors must **not** cause memory corruption, undefined behavior,
//! or any other safety violations, and callers must **not** rely on such outcomes.
//!
//! # Limitations and workarounds
//!
//! In some cases, you may need to explicitly annotate the parameter types in closures,
//! especially for adaptors that take generic functions.
//! This is due to current limitations in Rust’s type inference for closure parameters.
//!
//! Moreover, if you ever... [TODO]
//!
//! # Example
//!
//! Suppose we are building a tokenizer to process text for an NLP model.
//! We will skip all complicated details for now and simply collect every word we see.
//!
//! ```
//! use std::{ops::ControlFlow, collections::HashMap};
//! use better_collect::prelude::*;
//!
//! #[derive(Default)]
//! struct Tokenizer {
//!     indices: HashMap<String, usize>,
//!     words: Vec<String>,
//! }
//!
//! impl Tokenizer {
//!     fn tokenize(&self, sentence: &str) -> Vec<usize> {
//!         sentence
//!             .split_whitespace()
//!             .map(|word| self.indices.get(word).copied().unwrap_or(0))
//!             .collect()
//!     }
//! }
//!
//! impl Collector for Tokenizer {
//!     type Item = String;
//!     // For now, for simplicity, we just return the struct itself.
//!     type Output = Self;
//!
//!     fn collect(&mut self, word: Self::Item) -> ControlFlow<()> {
//!         self.indices
//!             .entry(word)
//!             .or_insert_with_key(|word| {
//!                 self.words.push(word.clone());
//!                 // Reserve index 0 for out-of-vocabulary words.
//!                 self.words.len()
//!             });
//!
//!         // Tokenizer never stops accumulating.
//!         ControlFlow::Continue(())
//!     }
//!
//!     fn finish(self) -> Self::Output {
//!         // Just return itself.
//!         self
//!     }
//! }
//!
//! let sentence = "the noble and the singer";
//! let tokenizer = sentence
//!     .split_whitespace()
//!     .map(String::from)
//!     .feed_into(Tokenizer::default());
//!
//! // "the" should only appear once.
//! assert_eq!(tokenizer.words, ["the", "noble", "and", "singer"]);
//! assert_eq!(tokenizer.tokenize("the singer and the swordswoman"), [1, 4, 3, 1, 0]);
//! ```
//!
//!
//! [`Break(())`]: std::ops::ControlFlow::Break

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
