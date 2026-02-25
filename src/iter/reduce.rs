use crate::collector::{Collector, CollectorBase, assert_collector};

use std::{fmt::Debug, ops::ControlFlow};

/// A collector that reduces all collected items into a single value
/// by repeatedly applying a reduction function.
///
/// If no items have been collected, its [`Output`](CollectorBase::Output) is `None`;
/// otherwise, it returns `Some` containing the result of the reduction.
///
/// This collector corresponds to [`Iterator::reduce()`].
///
/// # Examples
///
/// ```
/// use komadori::{prelude::*, iter::Reduce};
///
/// let mut collector = Reduce::new(|accum, num| accum + num);
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(3).is_continue());
/// assert!(collector.collect(5).is_continue());
///
/// assert_eq!(collector.finish(), Some(9));
/// ```
///
/// The output is `None` if no items were collected.
///
/// ```
/// use komadori::{prelude::*, iter::Reduce};
///
/// assert_eq!(Reduce::new(|accum: i32, num| accum + num).finish(), None);
/// ```
#[derive(Clone)]
pub struct Reduce<T, F> {
    accum: Option<T>,
    f: F,
}

impl<T, F> Reduce<T, F>
where
    F: FnMut(T, T) -> T,
{
    /// Crates a new instance of this collector with a given accumulator.
    #[inline]
    pub const fn new(f: F) -> Self {
        assert_collector::<_, T>(Self { accum: None, f })
    }
}

impl<T, F> CollectorBase for Reduce<T, F> {
    type Output = Option<T>;

    #[inline]
    fn finish(self) -> Self::Output {
        self.accum
    }
}

impl<T, F> Collector<T> for Reduce<T, F>
where
    F: FnMut(T, T) -> T,
{
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        if let Some(accum) = self.accum.take() {
            self.accum = Some((self.f)(accum, item));
        } else {
            self.accum = Some(item);
        };

        ControlFlow::Continue(())
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        self.accum = self
            .accum
            .take()
            .into_iter()
            .chain(items)
            .reduce(&mut self.f);

        ControlFlow::Continue(())
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        self.accum.into_iter().chain(items).reduce(self.f)
    }
}

impl<T: Debug, F> Debug for Reduce<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reduce")
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
        #[test]
        fn all_collect_methods(
            nums in propvec(any::<i32>(), ..=9),
        ) {
            all_collect_methods_impl(nums)?;
        }
    }

    fn all_collect_methods_impl(nums: Vec<i32>) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || nums.iter().copied(),
            collector_factory: || Reduce::new(|a, b| a ^ b),
            should_break_pred: |_| false,
            pred: |iter, output, remaining| {
                if iter.reduce(|a, b| a ^ b) != output {
                    Err(PredError::IncorrectOutput)
                } else if remaining.ne([]) {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector()
    }
}
