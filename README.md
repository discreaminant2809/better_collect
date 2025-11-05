# better_collect

Provides a composable, declarative way to consume an iterator.

If [`Iterator`] is the "source half" of data pipeline, [`Collector`] is the "sink half" of the pipeline.

In order words, [`Iterator`] describes *how to produce* data, and [`Collector`] describes *how to consume* it.

## Motivation

Suppose we are given an array of `i32` and we are asked to find its sum and maximum value.
What would be our approach?

- Approach 1: Two-pass

```rust
let nums = [1, 3, 2];
let sum: i32 = nums.into_iter().sum();
let max = nums.into_iter().max().unwrap();

assert_eq!(sum, 6);
assert_eq!(max, 3);
```

**Cons:** This performs two passes over the data, which is worse than one-pass in performance.
That is fine for arrays, but can be much worse for [`HashSet`], [`LinkedList`],
or... data from an IO stream.

- Approach 2: [`Iterator::fold`]

```rust
let nums = [1, 3, 2];
let (sum, max) = nums
    .into_iter()
    .fold((0, i32::MIN), |(sum, max), num| {
        (sum + num, max.max(num))
    });

assert_eq!(sum, 6);
assert_eq!(max, 3);
```

**Cons:** Not very declarative — the main logic is still kind of procedural. (Doing sum and max by ourselves)

This crate proposes a one-pass, declarative approach:

```rust
use better_collect::{
    Collector, RefCollector, BetterCollect,
    num::Sum, cmp::Max,
};

let nums = [1, 3, 2];
let (sum, max) = nums
    .into_iter()
    .better_collect(Sum::<i32>::new().then(Max::new()));

assert_eq!(sum, 6);
assert_eq!(max.unwrap(), 3);
```

This approach achieves both one-pass and declarative, while is also composable (more of this later).

This is only with integers. How about with a non-`Copy` type?

```rust
// Suppose we open a connection...
fn socket_stream() -> impl Iterator<Item = String> {
    ["the", "nobel", "and", "the", "singer"]
        .into_iter()
        .map(String::from)
}

// Task: Returns:
// - An array of data from the stream.
// - How many bytes were read.
// - The last-seen data.

// Usually, we're pretty much stuck with for-loop (tradition, `(try_)fold`, `(try_)for_each`).
// No common existing tools can help us here:
let mut received = vec![];
let mut byte_read = 0_usize;
let mut last_seen = None;

for data in socket_stream() {
    received.push(data.clone());
    byte_read += data.len();
    last_seen = Some(data);
}

let expected = (received, byte_read, last_seen);

// This crate's way:
use better_collect::{
    Collector, RefCollector, BetterCollect,
    Last, num::Sum,
};

let ((received, byte_read), last_seen) = socket_stream()
    .better_collect(
        vec![]
            .cloned()
            // Use `map_ref` so that our collector is a `RefCollector`
            // (only a `RefCollector` is then-able)
            .then(Sum::<usize>::new().map_ref(|data: &mut String| data.len()))
            .then(Last::new())
    );

assert_eq!((received, byte_read, last_seen), expected);
```

Very declarative! We describe what we want to collect.

You might think this is just like [`Iterator::unzip`], but this crate does a bit better:
it can split the data and feed separately **WITHOUT** additional allocation.

To demonstrate the difference, take this example:

```rust
use std::collections::HashSet;
use better_collect::{
    Collector, RefCollector, BetterCollect,
    string::ConcatString,
};

// Suppose we open a connection...
fn socket_stream() -> impl Iterator<Item = String> {
    ["the", "nobel", "and", "the", "singer"]
        .into_iter()
        .map(String::from)
}

// Task: Collect UNIQUE chunks of data and concatenate them.

// `Iterator::unzip`
let (chunks, concatenated_data): (HashSet<_>, String) = socket_stream()
    // Sad. We have to clone.
    // We can't take a reference, since the referenced data is returned too.
    .map(|chunk| (chunk.clone(), chunk))
    .unzip();

let unzip_way = (concatenated_data, chunks);

// Another approach is do two passes (collect to `Vec`, then iterate),
// which is still another allocation,
// or `Iterator::fold`, which's procedural.

// `Collector`
let collector_way = socket_stream()
    // No clone - the data flows smoothly.
    .better_collect(ConcatString::new().then(HashSet::new()));

assert_eq!(unzip_way, collector_way);
```

## Traits

Unlike [`std::iter`], this crate defines two main traits instead. Roughly:

```rust
use std::ops::ControlFlow;

pub trait Collector: Sized {
    type Item;
    type Output;

    fn collect(&mut self, item: Self::Item) -> ControlFlow<()>;
    fn finish(self) -> Self::Output;
}

pub trait RefCollector: Collector {
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()>;
}
```

[`Collector`] is similar to [`Extend`], but it also returns a [`ControlFlow`]
value to indicate whether it should stop accumulating items after a call to
[`collect()`].
This serves as a hint for adaptors like [`then()`] or [`chain()`]
to "vectorize" the remaining items to another collector.
In short, it is like a **composable** [`Extend`].

[`RefCollector`] is a collector that does not require ownership of an item
to process it.
This allows items to flow through multiple collectors without being consumed,
avoiding unnecessary cloning.
It powers [`then()`], which creates a pipeline of collectors,
letting each item pass through safely by reference until the final collector
takes ownership.

Finally, [`BetterCollect`] extends [`Iterator`] with the
[`better_collect()`] method, which feeds all items from an iterator
into a [`Collector`] and returns the collector’s result.
To use this method, the [`BetterCollect`] trait must be imported.

More details can be found in their respective documentation.

## Features

If the `std` feature is **not** enabled, the crate builds in `no_std` mode.

- **`alloc`** — Enables implementations for types in the [`alloc`] crate
  (e.g., [`Vec`], [`VecDeque`], [`BTreeSet`]).
- **`std`** *(default)* — Enables the `alloc` feature and implementations
  for [`std`]-only types (e.g., [`HashMap`]).

## Todos

- More detailed documentation.
- More adaptors (this crate currently only has common ones).
- Possibly foreign implementations for types in other crates.

[`Collector`]: https://docs.rs/better_collect/latest/better_collect/trait.Collector.html
[`RefCollector`]: https://docs.rs/better_collect/latest/better_collect/trait.RefCollector.html
[`BetterCollect`]: https://docs.rs/better_collect/latest/better_collect/trait.BetterCollect.html
[`collect()`]: https://docs.rs/better_collect/latest/better_collect/trait.Collector.html#tymethod.collect
[`better_collect()`]: https://docs.rs/better_collect/latest/better_collect/trait.BetterCollect.html#method.better_collect
[`chain()`]: https://docs.rs/better_collect/latest/better_collect/trait.Collector.html#method.chain
[`then()`]: https://docs.rs/better_collect/latest/better_collect/trait.RefCollector.html#method.then
[`Iterator`]: https://doc.rust-lang.org/1.90.0/std/iter/trait.Iterator.html
[`Extend`]: https://doc.rust-lang.org/1.90.0/std/iter/trait.Extend.html
[`Iterator::fold`]: https://doc.rust-lang.org/1.90.0/std/iter/trait.Iterator.html#method.fold
[`Iterator::unzip`]: https://doc.rust-lang.org/1.90.0/std/iter/trait.Iterator.html#method.unzip
[`std::iter`]: https://doc.rust-lang.org/std/iter/index.html
[`Vec`]: https://doc.rust-lang.org/1.90.0/std/vec/struct.Vec.html
[`HashSet`]: https://doc.rust-lang.org/1.90.0/std/collections/struct.HashSet.html
[`HashMap`]: https://doc.rust-lang.org/1.90.0/std/collections/struct.HashMap.html
[`LinkedList`]: https://doc.rust-lang.org/1.90.0/std/collections/struct.LinkedList.html
[`ControlFlow`]: https://doc.rust-lang.org/1.90.0/std/ops/enum.ControlFlow.html
[`alloc`]: https://doc.rust-lang.org/1.90.0/alloc/index.html
[`std`]: https://doc.rust-lang.org/1.90.0/std/index.html
[`VecDeque`]: https://doc.rust-lang.org/1.90.0/std/collections/struct.VecDeque.html
[`BTreeSet`]: https://doc.rust-lang.org/1.90.0/std/collections/struct.BTreeSet.html
