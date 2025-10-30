use std::{fmt::Debug, ops::ControlFlow};

use crate::{Collector, Fuse, RefCollector};

#[derive(Clone)]
pub struct Partition<CT, CF, F> {
    // `Fuse` is neccessary since we need to assess one's finishing state while assessing another,
    // like in `collect`.
    collector_if_true: Fuse<CT>,
    collector_if_false: Fuse<CF>,
    pred: F,
}

impl<CT, CF, F> Partition<CT, CF, F> {
    pub fn new(collector_if_true: CT, collector_if_false: CF, pred: F) -> Self {
        Self {
            collector_if_true: Fuse::new(collector_if_true),
            collector_if_false: Fuse::new(collector_if_false),
            pred,
        }
    }
}

// Put in a macro instead of function so that the short-circuit nature of `&&` is pertained.
macro_rules! cf_and {
    ($cf:expr, $pred:expr) => {
        // Can't swap, since we have to collect regardless.
        if $cf && $pred {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
}

impl<T, CT, CF, F> Collector<T> for Partition<CT, CF, F>
where
    CT: Collector<T>,
    CF: Collector<T>,
    F: FnMut(&mut T) -> bool,
{
    type Output = (CT::Output, CF::Output);

    fn collect(&mut self, mut item: T) -> ControlFlow<()> {
        if (self.pred)(&mut item) {
            cf_and!(
                self.collector_if_true.collect(item).is_break(),
                self.collector_if_false.finished()
            )
        } else {
            cf_and!(
                self.collector_if_false.collect(item).is_break(),
                self.collector_if_true.finished()
            )
        }
    }

    fn finish(self) -> Self::Output {
        (
            self.collector_if_true.finish(),
            self.collector_if_false.finish(),
        )
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        match items.try_for_each(|mut item| {
            #[allow(clippy::collapsible_else_if)] // we want it to be mirrored.
            if (self.pred)(&mut item) {
                if self.collector_if_true.collect(item).is_break() {
                    ControlFlow::Break(true)
                } else {
                    ControlFlow::Continue(())
                }
            } else {
                if self.collector_if_false.collect(item).is_break() {
                    ControlFlow::Break(false)
                } else {
                    ControlFlow::Continue(())
                }
            }
        }) {
            ControlFlow::Break(true) => {
                cf_and!(
                    self.collector_if_false
                        // Can't use `Iterator::filter` since it expects `&T`, not `&mut T` like us.
                        // `Iterator::filter_map` is lowkey great workaround in this case.
                        .collect_many(
                            items.filter_map(|mut item| (!(self.pred)(&mut item)).then_some(item)),
                        )
                        .is_break(),
                    self.collector_if_true.finished()
                )
            }
            ControlFlow::Break(false) => {
                cf_and!(
                    self.collector_if_true
                        .collect_many(
                            items.filter_map(|mut item| (self.pred)(&mut item).then_some(item)),
                        )
                        .is_break(),
                    self.collector_if_false.finished()
                )
            }
            ControlFlow::Continue(_) => ControlFlow::Continue(()),
        }
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = T>) -> Self::Output {
        let mut items = items.into_iter();

        match items.try_for_each(|mut item| {
            #[allow(clippy::collapsible_else_if)] // we want it to be mirrored.
            if (self.pred)(&mut item) {
                if self.collector_if_true.collect(item).is_break() {
                    ControlFlow::Break(true)
                } else {
                    ControlFlow::Continue(())
                }
            } else {
                if self.collector_if_false.collect(item).is_break() {
                    ControlFlow::Break(false)
                } else {
                    ControlFlow::Continue(())
                }
            }
        }) {
            ControlFlow::Break(true) => (
                self.collector_if_true.finish(),
                self.collector_if_false.collect_then_finish(
                    items.filter_map(|mut item| (!(self.pred)(&mut item)).then_some(item)),
                ),
            ),
            ControlFlow::Break(false) => (
                self.collector_if_true.collect_then_finish(
                    items.filter_map(|mut item| (self.pred)(&mut item).then_some(item)),
                ),
                self.collector_if_false.finish(),
            ),
            ControlFlow::Continue(_) => self.finish(),
        }
    }
}

impl<T, CT, CF, F> RefCollector<T> for Partition<CT, CF, F>
where
    CT: RefCollector<T>,
    CF: RefCollector<T>,
    F: FnMut(&mut T) -> bool,
{
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()> {
        if (self.pred)(item) {
            cf_and!(
                self.collector_if_true.collect_ref(item).is_break(),
                self.collector_if_false.finished()
            )
        } else {
            cf_and!(
                self.collector_if_false.collect_ref(item).is_break(),
                self.collector_if_true.finished()
            )
        }
    }
}

impl<CT: Debug, CF: Debug, F> Debug for Partition<CT, CF, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Partition")
            .field("collector_if_true", &self.collector_if_true)
            .field("collector_if_false", &self.collector_if_false)
            .finish()
    }
}
