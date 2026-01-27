use std::ops::ControlFlow;

use crate::collector::{Collector, CollectorBase};

/// Use [`Dropping`](crate::mem::Dropping).
#[deprecated(since = "0.4.0", note = "Use `Dropping`")]
#[derive(Clone, Debug, Default)]
pub struct Sink;

#[allow(deprecated)]
impl Sink {
    /// Creates a new instance of this collector.
    pub const fn new() -> Self {
        Sink
    }
}

#[allow(deprecated)]
impl CollectorBase for Sink {
    type Output = ();

    fn finish(self) -> Self::Output {}
}

#[allow(deprecated)]
impl<T> Collector<T> for Sink {
    #[inline]
    fn collect(&mut self, _item: T) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        items.into_iter().for_each(drop);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = T>) -> Self::Output {
        items.into_iter().for_each(drop);
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

    use crate::test_utils::{BasicCollectorTester, CollectorTesterExt, PredError};

    use super::*;

    proptest! {
        #[test]
        fn all_collect_methods(
            count in ..5_usize,
        ) {
            all_collect_methods_impl(count)?;
        }
    }

    fn all_collect_methods_impl(count: usize) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || std::iter::repeat_n(0, count),
            collector_factory: || {
                #[allow(deprecated)]
                {
                    Sink
                }
            },
            should_break_pred: |_| false,
            pred: |_, _, remaining| {
                if remaining.count() > 0 {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector()
    }
}
