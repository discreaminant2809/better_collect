//! [![Crates.io Version](https://img.shields.io/crates/v/better_collect.svg)](https://crates.io/crates/better_collect)
//! [![Docs.rs](https://img.shields.io/docsrs/better_collect)](https://docs.rs/better_collect)
//! [![GitHub Repo](https://img.shields.io/badge/github-better__collect-blue?logo=github)](https://github.com/discreaminant2809/better_collect.git)
//!
//! Provides a composable, declarative way to consume an iterator.
//!
//! If [`Iterator`] is the "source half" of data pipeline, [`Collector`] is the "sink half" of the pipeline.
//!
//! In order words, [`Iterator`] describes how to produce data, and [`Collector`] describes how to consume it.
//!
//! # Motivation
//!
//! Suppose we are given an array of `i32` and we are asked to find its sum and maximum value.
//! What would be our approach?
//!
//! - Approach 1: Two-pass
//!
//! ```
//! let nums = [1, 3, 2];
//! let sum: i32 = nums.into_iter().sum();
//! let max = nums.into_iter().max().unwrap();
//!
//! assert_eq!(sum, 6);
//! assert_eq!(max, 3);
//! ```
//!
//! **Cons:** This performs two passes over the data, which is worse than one-pass in performance.
//! That is fine for arrays, but can be much worse for [`HashSet`], [`LinkedList`],
//! or... data from an IO stream.
//!
//! - Approach 2: [`Iterator::fold()`]
//!
//! ```
//! let nums = [1, 3, 2];
//! let (sum, max) = nums
//!     .into_iter()
//!     .fold((0, i32::MIN), |(sum, max), num| {
//!         (sum + num, max.max(num))
//!     });
//!
//! assert_eq!(sum, 6);
//! assert_eq!(max, 3);
//! ```
//!
//! **Cons:** Not very declarative. The main logic is still kind of procedural.
//! (Doing sum and max by ourselves)
//!
//! - Approach 3: [`Iterator::inspect()`]
//!
//! ```
//! let nums = [1, 3, 2];
//! let mut sum = 0;
//! let max = nums
//!     .into_iter()
//!     .inspect(|i| sum += i)
//!     .max()
//!     .unwrap();
//!
//! assert_eq!(sum, 6);
//! assert_eq!(max, 3);
//! ```
//!
//! **Cons:** This approach has multiple drawbacks:
//!
//! - If the requirement changes to "calculate sum and find any negative value,"
//!   this approach may produce incorrect results.
//!   The "any" logic may short-circuit on finding the desired value,
//!   preventing the "sum" logic from summing every value.
//!   It is possible that we can rearrange so that the "any" logic goes first,
//!   but if the requirement changes to "find any negative value and even value,"
//!   we cannot escape.
//!
//! - The state is kept outside. Now the iterator cannot go anywhere else
//!   (e.g. sending to another thread, sending through a channel).
//!
//! - Very unintuitive and hack-y (hard to reason about).
//!
//! - And most importantly, not declarative enough.
//!
//! This crate proposes a one-pass, declarative approach:
//!
//! ```
//! use better_collect::{prelude::*, cmp::Max};
//!
//! let nums = [1, 3, 2];
//! let (sum, max) = nums
//!     .into_iter()
//!     .feed_into(i32::adding().tee(Max::new()));
//!
//! assert_eq!(sum, 6);
//! assert_eq!(max.unwrap(), 3);
//! ```
//!
//! This approach achieves both one-pass and declarative, while is also composable (more of this later).
//!
//! This is only with integers. How about with a non-`Copy` type?
//!
//! ```
//! // Suppose we open a connection...
//! fn socket_stream() -> impl Iterator<Item = String> {
//!     ["the", "noble", "and", "the", "singer"]
//!         .into_iter()
//!         .map(String::from)
//! }
//!
//! // Task: Returns:
//! // - An array of data from the stream.
//! // - How many bytes were read.
//! // - The last-seen data.
//!
//! // Usually, we're pretty much stuck with for-loop
//! // (tradition, `(try_)fold`, `(try_)for_each`).
//! // No common existing tools can help us here:
//! let mut byte_read = 0_usize;
//! let mut received = vec![];
//! let mut last_seen = None;
//!
//! for data in socket_stream() {
//!     byte_read += data.len();
//!     received.push(data.clone());
//!     last_seen = Some(data);
//! }
//!
//! let expected = (byte_read, received, last_seen);
//!
//! // This crate's way:
//! use better_collect::{prelude::*, iter::Last};
//!
//! let ((byte_read, received), last_seen) = socket_stream()
//!     .feed_into(
//!         usize::adding()
//!             .map({
//!                 let f = |data: &mut String| data.len();
//!                 f
//!             })
//!             .tee_funnel(vec![])
//!             .tee_clone(Last::new())
//!     );
//!
//! assert_eq!((byte_read, received, last_seen), expected);
//! ```
//!
//! Very declarative! We describe what we want to collect.
//!
//! You might think this is just like [`Iterator::unzip()`]...
//!
//! Consider this example:
//!
//! ```
//! use std::collections::HashSet;
//! use better_collect::prelude::*;
//!
//! // Suppose we open a connection...
//! fn socket_stream() -> impl Iterator<Item = String> {
//!     ["the", "noble", "and", "the", "singer"]
//!         .into_iter()
//!         .map(String::from)
//! }
//!
//! // Task: Collect UNIQUE chunks of data and concatenate them.
//!
//! // `Iterator::unzip`
//! let unzip_way: (String, HashSet<_>) = socket_stream()
//!     // Sad. We have to clone.
//!     // We can't take a reference, since the referenced data is returned too.
//!     .map(|chunk| (chunk.clone(), chunk))
//!     .unzip();
//!
//! // Another approach is do two passes (collect to `Vec`, then iterate),
//! // which is still another allocation,
//! // or `Iterator::fold`, which's procedural.
//!
//! // `Collector`
//! let collector_way = socket_stream()
//!     // No clone. The data flows smoothly.
//!     .feed_into(
//!         String::new()
//!             .into_concat()
//!             .map({
//!                 fn f(s: &mut String) -> &str {
//!                     &s[..]
//!                 }
//!                 f
//!             })
//!             .tee_funnel(HashSet::new())
//!     );
//!
//! assert_eq!(unzip_way, collector_way);
//! ```
//!
//! # Crate stucture
//!
//! Modules in this crate mirror those in the standard library, because this crate
//! extends many types there. There is also `collector` which
//! contains collector functionalities that work behind [`feed_into()`],
//! and `prelude` which re-exports commons items for easier use.
//!
//! It is recommended to read the documentation of `collector` next
//! if you want to delve into how collectors work.
//!
//! # Features
//!
//! - **`alloc`** — Enables collectors and implementations for types in the
//!   [`alloc`] crate (e.g., [`Vec`], [`VecDeque`], [`BTreeSet`]).
//!
//! - **`std`** *(default)* — Enables the `alloc` feature and implementations
//!   for [`std`]-only types (e.g., [`HashMap`]).
//!   When this feature is disabled, the crate builds in `no_std` mode.
//!
//! - **`unstable`** — Enables experimental and unstable features.
//!   Items gated behind this feature do **not** follow normal semver guarantees
//!   and may change or be removed at any time.
//!
//!   Although the crate as a whole is technically still experimental, the items under
//!   `unstable` are even more experimental, and it is generally
//!   discouraged to use them until their designs are finalized and not
//!   under this flag anymore.
//!
//! [`Collector`]: crate::collector::Collector
//! [`feed_into()`]: crate::iter::IteratorExt::feed_into
//! [`HashSet`]: std::collections::HashSet
//! [`HashMap`]: std::collections::HashMap
//! [`LinkedList`]: std::collections::LinkedList
//! [`ControlFlow`]: core::ops::ControlFlow
//! [`VecDeque`]: std::collections::VecDeque
//! [`BTreeSet`]: std::collections::BTreeSet

#![forbid(missing_docs)]
#![cfg_attr(test, deny(deprecated))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
// To make doc examples in sync (prevent accidental deprecated items usage in doc).
#![doc(test(attr(deny(deprecated))))]

#[cfg(any(doc, all(feature = "alloc", not(feature = "std"))))]
extern crate alloc;

#[cfg(not(feature = "std"))]
extern crate core as std;

// #[cfg(feature = "unstable")]
// pub mod aggregate;
pub mod cmp;
#[cfg(feature = "alloc")]
pub mod collections;
pub mod collector;
pub mod iter;
pub mod mem;
pub mod num;
pub mod ops;
pub mod prelude;
pub mod slice;
#[cfg(feature = "alloc")]
pub mod string;
#[cfg(feature = "std")]
pub mod sync;
pub mod unit;
#[cfg(feature = "alloc")]
pub mod vec;

#[cfg(all(test, feature = "std"))]
mod test_utils;

#[cfg(feature = "unstable")]
#[inline(always)]
const fn assert_iterator<I: Iterator>(iterator: I) -> I {
    iterator
}
