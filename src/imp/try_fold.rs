use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, RefCollector, assert_collector, assert_ref_collector};

/// A [`Collector`] that accumulates items using a function
/// as long as the function returns successfully.
///
/// This collector corresponds to [`Iterator::try_fold()`], except that
/// the accumulated value is mutated in place, and its result type
/// is **not** wrapped in a control-flow container.
///
/// Currently, it only supports [`ControlFlow`] as the function’s return type.
/// More types may be supported once the [`Try`](std::ops::Try) trait is stabilized.
///
/// This collector has a `Ref` counterpart created by [`new_ref()`](TryFold::new_ref).
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, TryFold};
/// use std::ops::ControlFlow;
///
/// let mut collector = TryFold::new(0_i8, |sum, num| {
///     match sum.checked_add(num) {
///         Some(new_sum) => {
///             *sum = new_sum;
///             ControlFlow::Continue(())
///         }
///         None => ControlFlow::Break(())
///     }
/// });
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert_eq!(collector.finish(), 6);
/// ```
///
/// Short-circuiting:
///
/// ```
/// use better_collect::{prelude::*, TryFold};
/// use std::ops::ControlFlow;
///
/// let mut collector = TryFold::new(0_i8, |sum, num| {
///     match sum.checked_add(num) {
///         Some(new_sum) => {
///             *sum = new_sum;
///             ControlFlow::Continue(())
///         }
///         None => ControlFlow::Break(())
///     }
/// });
///
/// assert!(collector.collect(60).is_continue());
/// assert!(collector.collect(60).is_continue());
///
/// // The addition operation overflows.
/// assert!(collector.collect(60).is_break());
///
/// assert_eq!(collector.finish(), 120);
/// ```
pub struct TryFold<A, T, F> {
    accum: A,
    f: F,
    // Needed, or else the compiler will complain about "unconstraint generics."
    // Since we use `T` in the function params, it's logical to use `PhantomData` like this.
    _marker: PhantomData<fn(T)>,
}

/// A [`RefCollector`] that accumulates items by mutable reference using a function
/// as long as the function returns successfully.
///
/// This is the `Ref` counterpart and shares the same semantics as [`TryFold`].
/// Ses its documentation for more.
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, TryFold};
/// use std::ops::ControlFlow;
///
/// let (total_len, concatenated) = ["abc", "de", "fgh"]
///     .into_iter()
///     .map(String::from)
///     .better_collect(
///         TryFold::new_ref(0, |total_len, s: &mut String| {
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
/// use better_collect::{prelude::*, TryFold};
/// use std::ops::ControlFlow;
///
/// let (concatenated_till_empty, concatenated) = ["abc", "de", "", "fgh"]
///     .into_iter()
///     .map(String::from)
///     .better_collect(
///         TryFold::new_ref("".to_owned(), |concatenated_till_empty, s: &mut String| {
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
pub struct TryFoldRef<A, T, F> {
    accum: A,
    f: F,
    // Needed, or else the compiler will complain about "unconstraint generics."
    // Since we use `&mut T` in the function params, it's logical to use `PhantomData` like this.
    _marker: PhantomData<fn(&mut T)>,
}

impl<A, T, F> TryFold<A, T, F> {
    /// Creates a new instance of this collector with an initial value and an accumulator.
    #[inline]
    pub const fn new(init: A, f: F) -> Self
    where
        F: FnMut(&mut A, T) -> ControlFlow<()>,
    {
        assert_collector(TryFold {
            accum: init,
            f,
            _marker: PhantomData,
        })
    }

    /// Creates a new instance of the `Ref` counterpart of this collector
    /// with an initial value and an accumulator.
    #[inline]
    pub const fn new_ref(accum: A, f: F) -> TryFoldRef<A, T, F>
    where
        F: FnMut(&mut A, &mut T) -> ControlFlow<()>,
    {
        assert_ref_collector(TryFoldRef {
            accum,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, T, F> Collector for TryFold<A, T, F>
where
    F: FnMut(&mut A, T) -> ControlFlow<()>,
{
    type Item = T;
    type Output = A;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}

impl<A: Clone, T, F: Clone> Clone for TryFold<A, T, F> {
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

impl<A: Debug, T, F> Debug for TryFold<A, T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TryFold")
            .field("accum", &self.accum)
            .finish()
    }
}

impl<A, T, F> Collector for TryFoldRef<A, T, F>
where
    F: FnMut(&mut A, &mut T) -> ControlFlow<()>,
{
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

impl<A, T, F> RefCollector for TryFoldRef<A, T, F>
where
    F: FnMut(&mut A, &mut T) -> ControlFlow<()>,
{
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
