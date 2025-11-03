//! Provides a composable, declarative way to consume an iterator.
//!
//! If [`Iterator`] is the "source half" of data pipeline, [`Collector`] is the "sink half" of the pipeline.
//!
//! In order words, [`Iterator`] describes *how to produce* data, and [`Collector`] describes *how to consume* it.
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
//! - Approach 2: [`Iterator::fold`]
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
//! **Cons:** Not very declarative — the main logic is still kind of procedural. (Doing sum and max by ourselves)
//!
//! This crate proposes a one-pass, declarative approach:
//!
//! ```
//! use better_collect::{
//!     Collector, RefCollector, BetterCollect,
//!     num::Sum, cmp::Max,
//! };
//!
//! let nums = [1, 3, 2];
//! let (sum, max) = nums
//!     .into_iter()
//!     .better_collect(Sum::<i32>::new().then(Max::new()));
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
//!     ["12", "34", "not an integer", "56"]
//!         .into_iter()
//!         .map(String::from)
//! }
//!
//! // Task: Returns:
//! // - An array of `i32` successfully parsed from the stream (skip all unparsable data).
//! // - How many bytes were read.
//! // - The last-seen data.
//!
//! // Usually, we're pretty much stuck with for-loop (tradition, `(try_)fold`, `(try_)for_each`).
//! // No common existing tools can help us here:
//! let mut received = vec![];
//! let mut byte_read = 0_usize;
//! let mut last_seen = None;
//!
//! for data in socket_stream() {
//!     received.push(data.clone());
//!     byte_read += data.len();
//!     last_seen = Some(data);
//! }
//!
//! let expected = (received, byte_read, last_seen);
//!
//! // This crate's way:
//! use better_collect::{
//!     Collector, RefCollector, BetterCollect,
//!     Last, num::Sum,
//! };
//!
//! let ((received, byte_read), last_seen) = socket_stream()
//!     .better_collect(
//!         vec![]
//!             .cloned()
//!             // Use `map_ref` so that our collector is a `RefCollector`
//!             // (only a `RefCollector` is then-able)
//!             .then(Sum::<usize>::new().map_ref(|data: &mut String| data.len()))
//!             .then(Last::new())
//!     );
//!
//! assert_eq!((received, byte_read, last_seen), expected);
//! ```
//!
//! Very declarative! We describe what we want to collect.
//!
//! You might think this is just like [`Iterator::unzip`], but this crate does a bit better:
//! it can split the data and feed separately **WITHOUT** additional allocation.
//!
//! To demonstrate the difference, take this example:
//!
//! ```
//! use std::collections::HashSet;
//! use better_collect::{
//!     Collector, RefCollector, BetterCollect,
//!     string::ConcatString,
//! };
//!
//! // Suppose we open a connection...
//! fn socket_stream() -> impl Iterator<Item = String> {
//!     ["a", "b", "c", "b"]
//!         .into_iter()
//!         .map(String::from)
//! }
//!
//! // Task: Collect UNIQUE chunks of data and concatenate them.
//!
//! // `Iterator::unzip`
//! let (chunks, concatenated_data): (HashSet<_>, String) = socket_stream()
//!     // Sad. We have to clone.
//!     // We can't take a reference, since the referenced data is returned too.
//!     .map(|chunk| (chunk.clone(), chunk))
//!     .unzip();
//!
//! let unzip_way = (concatenated_data, chunks);
//!
//! // Another approach is do two passes (collect to `Vec`, then iterate),
//! // which is still another allocation,
//! // or `Iterator::fold`, which's procedural.
//!
//! // `Collector`
//! let collector_way = socket_stream()
//!     // No clone - the data flows smoothly.
//!     .better_collect(ConcatString::new().then(HashSet::new()));
//!
//! assert_eq!(unzip_way, collector_way);
//! ```
//!
//! # Traits
//!
//! Unlike [`std::iter`], this crate has two main traits instead. Roughtly:
//!
//! ```no_run
//! use std::ops::ControlFlow;
//!
//! pub trait Collector: Sized {
//!     type Item;
//!     type Output;
//!
//!     fn collect(&mut self, item: Self::Item) -> ControlFlow<()>;
//!     fn finish(self) -> Self::Output;
//! }
//!
//! pub trait RefCollector: Collector {
//!     fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()>;
//! }
//! ```
//!
//! [`Collector`] is akin to [`Extend`], except it also returns [`ControlFlow`] to hint
//! whether it permanently stops receiving items after calling [`collect`](Collector::collect).
//! It is a helpful hint for some adaptors (e.g. [`Then`], [`Chain`]) to "vectorize" the rest of
//! the items to another collector. After returning ``
//!
//! [`RefCollector`] is a collector that does not need the ownership of an item to collect it.
//! The reason this trait exists is to allow an item to pass down through every collector without
//! any of them consumes it. This allows multiple consumptions from just one item instead of cloning it
//! to each collector, which is one of the main ideas of this crate. [`RefCollector`] powers [`then`](RefCollector::then),
//! which establishes a pipeline of collectors and lets an item from the iterator pass down safely without being
//! collected by ownership midway, to the last one when it is allowed to take the ownership of the item.
//!
//! Finally, [`BetterCollect`] adds a method [`better_collect`](BetterCollect::better_collect) to [`Iterator`]
//! to "use" [`Collector`]. It extracts items from the iterator to the collector, then makes the collector
//! produces the result. The trait has to be imported to be able to use the method.
//!
//! More detail can be found in their respected docs.
//!
//! # Features
//!
//! - `alloc`: Enables implementations for types in the [`alloc`] crate (e.g. [`Vec`], [`VecDeque`], [`BTreeSet`]).
//! - `std`: Enables the `alloc` feature, and implementations for [`std`]-only types (e.g. [`HashSet`]).
//!
//! # Todos
//!
//! - More detailed documentation.
//! - More adaptors (this crate currently only has common ones).
//! - Possibly foreign implementations for types in other crates.
//!
//! [`Iterator`]: std::iter::Iterator
//! [`Iterator::fold`]: std::iter::Iterator::fold
//! [`Iterator::unzip`]: std::iter::Iterator::unzip
//! [`std::iter`]: std::iter
//! [`HashSet`]: std::collections::HashSet
//! [`LinkedList`]: std::collections::LinkedList
//! [`ControlFlow`]: core::ops::ControlFlow
//! [`alloc`]: https://doc.rust-lang.org/1.90.0/alloc/index.html
//! [`std`]: std
//! [`VecDeque`]: std::collections::VecDeque
//! [`BTreeSet`]: std::collections::BTreeSet

// #![forbid(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(not(feature = "std"))]
extern crate core as std;

mod adaptors;
mod imp;
mod traits;

pub use adaptors::*;
pub use imp::*;
pub use traits::*;

#[inline(always)]
fn assert_collector<C: Collector>(collector: C) -> C {
    collector
}

#[inline(always)]
fn assert_ref_collector<C: RefCollector>(collector: C) -> C {
    collector
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::{BetterCollect, Collector, RefCollector};

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::{string::String, vec};

    #[cfg(feature = "alloc")]
    #[test]
    fn then() {
        let arr = [1, 2, 3];
        let (arr1, arr2) = arr.into_iter().better_collect(vec![].then(vec![]));
        assert_eq!(arr1, arr);
        assert_eq!(arr2, arr);

        let arr = ["1", "2", "3"];
        let (arr1, arr2) = ["1", "2", "3"]
            .into_iter()
            .map(String::from)
            .better_collect(vec![].cloned().then(vec![]));
        assert_eq!(arr1, arr);
        assert_eq!(arr2, arr);
    }
}
