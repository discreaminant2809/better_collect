use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

use crate::{Collector, assert_collector};

/// A [`Collector`] that accumulates items using a function
/// as long as the function returns successfully.
///
/// This collector corresponds to [`Iterator::try_fold()`], except that
/// the accumulator is mutated in place, and its result type
/// is **not** wrapped in a control-flow container.
///
/// Currently, it only supports [`ControlFlow`] as the function’s return type.
/// More types may be supported once the [`Try`](std::ops::Try) trait is stabilized.
///
/// Since it does **not** implement [`RefCollector`], this collector should be used
/// as the **final collector** in a [`then`] chain, or adapted into a [`RefCollector`]
/// using the appropriate adaptor.
/// If you find yourself writing `TryFold::new(...).cloned()` or `TryFold::new(...).copied()`,
/// consider using [`TryFoldRef`](crate::TryFoldRef) instead, which avoids unnecessary cloning.
///
/// # Examples
///
/// ```
/// use better_collect::{Collector, TryFold};
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
/// use better_collect::{Collector, TryFold};
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
///
/// [`RefCollector`]: crate::RefCollector
/// [`then`]: crate::RefCollector::then
pub struct TryFold<A, T, F> {
    accum: A,
    f: F,
    // Needed, or else the compiler will complain about "unconstraint generics."
    // Since we use `T` in the function params, it's logical to use `PhantomData` like this.
    _marker: PhantomData<fn(T)>,
}

impl<A, T, F: FnMut(&mut A, T) -> ControlFlow<()>> TryFold<A, T, F> {
    /// Creates a new instance of this collector with an initial value and an accumulator.
    #[inline]
    pub const fn new(init: A, f: F) -> Self {
        assert_collector(TryFold {
            accum: init,
            f,
            _marker: PhantomData,
        })
    }
}

impl<A, T, F: FnMut(&mut A, T) -> ControlFlow<()>> Collector for TryFold<A, T, F> {
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
