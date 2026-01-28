use std::{cmp::Ordering, ops::ControlFlow};

use super::{MaxBy, MaxByKey};

use crate::collector::{Collector, CollectorBase, assert_collector};

/// A [`Collector`] that computes the maximum value among the items it collects.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the maximum item otherwise.
///
/// This collector corresponds to [`Iterator::max()`].
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, cmp::Max};
///
/// let mut collector = Max::new();
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(3).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(5).is_continue());
/// assert!(collector.collect(3).is_continue());
///
/// assert_eq!(collector.finish(), Some(5));
/// ```
///
/// The output is `None` if no items were collected.
///
/// ```
/// use better_collect::{prelude::*, cmp::Max};
///
/// assert_eq!(Max::<i32>::new().finish(), None);
/// ```
#[derive(Debug, Clone)]
pub struct Max<T> {
    // For `Debug` impl used by `MaxByKey`.
    pub(super) max: Option<T>,
}

impl<T> Max<T> {
    /// Creates a new instance of this collector.
    #[inline]
    pub const fn new() -> Self
    where
        T: Ord,
    {
        assert_collector(Self { max: None })
    }

    /// Creates a new instance of [`MaxBy`] with a given comparison function.
    #[inline]
    pub const fn by<F>(f: F) -> MaxBy<T, F>
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        #[allow(deprecated)]
        assert_collector(MaxBy::new(f))
    }

    /// Creates a new instance of [`MaxByKey`] with a given key-extraction function.
    #[inline]
    pub const fn by_key<K, F>(f: F) -> MaxByKey<T, K, F>
    where
        K: Ord,
        F: FnMut(&T) -> K,
    {
        #[allow(deprecated)]
        assert_collector(MaxByKey::new(f))
    }
}

impl<T: Ord> Default for Max<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CollectorBase for Max<T> {
    type Output = Option<T>;

    #[inline]
    fn finish(self) -> Self::Output {
        self.max
    }
}

impl<T: Ord> Collector<T> for Max<T> {
    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        // Because it's `Max`, if `max` is a `None` then it's always smaller than a `Some`.
        // Doesn't work on `Min`, however.
        // Be careful to preserve the semantics of `Iterator::max` that if there are
        // more than one maximum values, the last one is chosen.
        self.max = self.max.take().max(Some(item));
        ControlFlow::Continue(())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.max = self.max.take().into_iter().chain(items).max();
        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.max.into_iter().chain(items).max()
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use std::cmp::Ordering;

    use proptest::collection::vec as propvec;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

    use crate::cmp::Max;
    use crate::test_utils::{BasicCollectorTester, CollectorTesterExt, PredError};

    proptest! {
        #[test]
        fn all_collect_methods_max(
            nums in propvec(any::<i32>(), ..5),
        ) {
            all_collect_methods_max_impl(nums)?;
        }
    }

    fn all_collect_methods_max_impl(nums: Vec<i32>) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || nums.iter().copied(),
            collector_factory: || Max::new(),
            should_break_pred: |_| false,
            pred: |iter, output, remaining| {
                if iter.max() != output {
                    Err(PredError::IncorrectOutput)
                } else if remaining.next().is_some() {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector()
    }

    proptest! {
        #[test]
        fn all_collect_methods_max_by(
            nums in propvec(any_complex(), ..5),
        ) {
            all_collect_methods_max_by_impl(nums)?;
        }
    }

    fn all_collect_methods_max_by_impl(nums: Vec<Complex>) -> TestCaseResult {
        // Suppose we compare them by the imaginary part first, then the real part.
        fn comparator(a: &Complex, b: &Complex) -> Ordering {
            let cmp_im = a.im.cmp(&b.im);
            if cmp_im.is_ne() {
                cmp_im
            } else {
                a.re.cmp(&b.re)
            }
        }

        BasicCollectorTester {
            iter_factory: || nums.iter().copied(),
            collector_factory: || Max::by(comparator),
            should_break_pred: |_| false,
            pred: |iter, output, remaining| {
                if iter.max_by(comparator) != output {
                    Err(PredError::IncorrectOutput)
                } else if remaining.next().is_some() {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector()
    }

    proptest! {
        #[test]
        fn all_collect_methods_max_by_key(
            nums in propvec(any_complex(), ..5),
        ) {
            all_collect_methods_max_by_key_impl(nums)?;
        }
    }

    fn all_collect_methods_max_by_key_impl(nums: Vec<Complex>) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || nums.iter().copied(),
            collector_factory: || Max::by_key(Complex::sqr_abs),
            should_break_pred: |_| false,
            pred: |iter, output, remaining| {
                if iter.max_by_key(Complex::sqr_abs) != output {
                    Err(PredError::IncorrectOutput)
                } else if remaining.next().is_some() {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector()
    }

    /// Helper struct for testing `MaxBy` and `MaxByKey`.
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    struct Complex {
        re: i32,
        im: i32,
    }

    impl Complex {
        pub fn sqr_abs(&self) -> i32 {
            // By restricting to `i16`, we guarantee that
            // this operation never overflows.
            // 2 * i16::MAX^2 < i32::MAX.
            self.re.pow(2) + self.im.pow(2)
        }
    }

    prop_compose! {
        fn any_complex()(
            // Restrict to `i16` because we're gonna calculate its squared abs.
            re in any::<i16>().prop_map_into::<i32>(),
            im in any::<i16>().prop_map_into::<i32>(),
        ) -> Complex {
            Complex { re, im }
        }
    }
}
