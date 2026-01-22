use std::{fmt::Debug, ops::ControlFlow};

use crate::collector::{Collector, CollectorBase, assert_collector};

/// A [`Collector`] that accumulates items using a function
/// as long as the function returns successfully.
///
/// This collector corresponds to [`Iterator::try_fold()`], except that
/// the accumulated value is mutated in place, and its result type
/// is not wrapped in a control-flow container.
///
/// Currently, it only supports [`ControlFlow`] as the function’s return type.
/// More types may be supported once the [`Try`](std::ops::Try) trait is stabilized.
///
/// This collector has a `Ref` counterpart created by [`new_ref()`](TryFold::new_ref).
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, iter::TryFold};
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
/// use better_collect::{prelude::*, iter::TryFold};
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
#[derive(Clone)]
pub struct TryFold<A, F> {
    accum: A,
    f: F,
}

impl<A, F> TryFold<A, F> {
    /// Creates a new instance of this collector with an initial value and an accumulator.
    #[inline]
    pub const fn new<T>(init: A, f: F) -> Self
    where
        F: FnMut(&mut A, T) -> ControlFlow<()>,
    {
        assert_collector::<_, T>(TryFold { accum: init, f })
    }
}

impl<A, F> CollectorBase for TryFold<A, F> {
    type Output = A;

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}

impl<A, T, F> Collector<T> for TryFold<A, F>
where
    F: FnMut(&mut A, T) -> ControlFlow<()>,
{
    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        (self.f)(&mut self.accum, item)
    }

    // The default implementations for `collect_many` and `collect_then_finish` are sufficient.
}

impl<A: Debug, F> Debug for TryFold<A, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TryFold")
            .field("accum", &self.accum)
            .finish()
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use proptest::collection::vec as propvec;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

    use crate::test_utils::{BasicCollectorTester, CollectorTesterExt, PredError};

    use super::*;

    proptest! {
        /// [`TryFold`](super::TryFold)
        #[test]
        fn all_collect_methods(
            nums in propvec(any::<u8>(), ..=9),
        ) {
            all_collect_methods_impl(nums)?;
        }

        /// [`TryFoldRef`](super::TryFoldRef)
        #[test]
        fn all_collect_methods_ref(
            nums in propvec(any::<u8>(), ..=5),
        ) {
            all_collect_methods_ref_impl(nums)?;
        }
    }

    fn all_collect_methods_impl(nums: Vec<u8>) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || nums.iter().copied(),
            collector_factory: || TryFold::new(Some(0_u8), collector_closure),
            should_break_pred: |iter| iter_output(iter).is_none(),
            pred: |mut iter, output, remaining| {
                let expected = iter_output(&mut iter);

                if expected != output {
                    Err(PredError::IncorrectOutput)
                } else if iter.ne(remaining) {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector()
    }

    fn all_collect_methods_ref_impl(nums: Vec<u8>) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || nums.iter().copied(),
            collector_factory: || {
                TryFold::new_ref(Some(0_u8), |sum, &mut num| collector_closure(sum, num))
            },
            should_break_pred: |iter| iter_output(iter).is_none(),
            pred: |mut iter, output, remaining| {
                let expected = iter_output(&mut iter);

                if expected != output {
                    Err(PredError::IncorrectOutput)
                } else if iter.ne(remaining) {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_ref_collector()
    }

    fn collector_closure(sum: &mut Option<u8>, num: u8) -> ControlFlow<()> {
        let curr = sum.expect("the correct usage is not to collect again");

        *sum = curr.checked_add(num);
        if sum.is_none() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn iter_output(iter: impl IntoIterator<Item = u8>) -> Option<u8> {
        iter.into_iter().try_fold(0_u8, u8::checked_add)
    }
}
