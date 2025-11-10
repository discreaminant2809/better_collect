use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_ref_collector};

/// A [`Collector`] that accumulates items by mutable reference using a function
/// as long as the function returns successfully.
///
/// This collector corresponds to [`Iterator::try_fold()`], except that
/// the accumulated value is mutated in place, and its result type
/// is **not** wrapped in a control-flow container.
///
/// Currently, it only supports [`ControlFlow`] as the function’s return type.
/// More types may be supported once the [`Try`](std::ops::Try) trait is stabilized.
///
/// Unlike [`TryFold`](crate::TryFold), this adaptor only receives a mutable reference to each item.
/// Because of that, it can be used **in the middle** of a [`then`] chain,
/// since it is a [`RefCollector`].
/// While it can also appear at the end of the chain, consider using [`TryFold`](crate::TryFold)
/// there instead for better clarity.
///
/// # Examples
///
/// ```
/// use better_collect::{
///     BetterCollect, RefCollector,
///     TryFoldRef, string::ConcatString,
/// };
/// use std::ops::ControlFlow;
///
/// let (total_len, concatenated) = ["abc", "de", "fgh"]
///     .into_iter()
///     .map(String::from)
///     .better_collect(
///         TryFoldRef::new(0, |total_len, s: &mut String| {
///             *total_len += s.len();
///             ControlFlow::Continue(())
///         })
///         .then(ConcatString::new())
///     );
///
/// assert_eq!(total_len, 8);
/// assert_eq!(concatenated, "abcdefgh");
/// ```
///
/// Short-circuiting:
///
/// ```
/// use better_collect::{
///     BetterCollect, RefCollector,
///     TryFoldRef, string::ConcatString,
/// };
/// use std::ops::ControlFlow;
///
/// let (concatenated_till_empty, concatenated) = ["abc", "de", "", "fgh"]
///     .into_iter()
///     .map(String::from)
///     .better_collect(
///         TryFoldRef::new("".to_owned(), |concatenated_till_empty, s: &mut String| {
///             if s.is_empty() {
///                 ControlFlow::Break(())
///             } else {
///                 *concatenated_till_empty += s;
///                 ControlFlow::Continue(())
///             }
///         })
///         .then(ConcatString::new())
///     );
///
/// assert_eq!(concatenated_till_empty, "abcde");
/// assert_eq!(concatenated, "abcdefgh");
/// ```
///
/// [`RefCollector`]: crate::RefCollector
/// [`then`]: crate::RefCollector::then
pub struct TryFoldRef<A, T, F> {
    accum: A,
    f: F,
    // Needed, or else the compiler will complain about "unconstraint generics."
    // Since we use `&mut T` in the function params, it's logical to use `PhantomData` like this.
    _marker: PhantomData<fn(&mut T)>,
}

impl<A, T, F: FnMut(&mut A, &mut T) -> ControlFlow<()>> TryFoldRef<A, T, F> {
    /// Creates a new instance of this collector with an initial value and an accumulator.
    #[inline]
    pub const fn new(accum: A, f: F) -> Self {
        assert_ref_collector(TryFoldRef {
            accum,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, T, F: FnMut(&mut A, &mut T) -> ControlFlow<()>> Collector for TryFoldRef<A, T, F> {
    type Item = T;
    type Output = A;

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        self.collect_ref(&mut item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}

impl<A, T, F: FnMut(&mut A, &mut T) -> ControlFlow<()>> RefCollector for TryFoldRef<A, T, F> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item)
    }
}

impl<A: Clone, T, F: Clone> Clone for TryFoldRef<A, T, F> {
    fn clone(&self) -> Self {
        Self {
            accum: self.accum.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.accum.clone_from(&source.accum);
        self.f.clone_from(&source.f);
    }
}

impl<A: Debug, T, F> Debug for TryFoldRef<A, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TryFoldRef")
            .field("accum", &self.accum)
            .finish()
    }
}
