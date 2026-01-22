use std::ops::ControlFlow;

use crate::collector::{Collector, CollectorBase};

/// A [`RefCollector`] that collects items... but no one knows where they go.
///
/// All we know is that it relentlessly consumes them, never to be seen again.
///
/// This collector is the counterpart of [`std::iter::empty()`], just like
/// [`std::io::sink()`] and [`std::io::empty()`].
///
/// # Examples
///
/// It collected the example. Nothing to show.
#[derive(Clone, Debug, Default)]
pub struct Sink;

impl CollectorBase for Sink {
    type Output = ();

    fn finish(self) -> Self::Output {}
}

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
            collector_factory: Sink::new,
            should_break_pred: |_| false,
            pred: |_, _, remaining| {
                if remaining.count() > 0 {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_ref_collector()
    }
}
