use std::{fmt::Debug, ops::ControlFlow};

use crate::{Collector, RefCollector};

/// A [`Collector`] that accumulates items as long as a predicate returns `true`.
///
/// This `struct` is created by [`Collector::take_while()`]. See its documentation for more.
#[derive(Clone)]
pub struct TakeWhile<C, F> {
    collector: C,
    pred: Option<F>,
}

impl<C, F> TakeWhile<C, F> {
    pub(crate) fn new(collector: C, pred: F) -> Self {
        Self {
            collector,
            pred: Some(pred),
        }
    }
}

impl<C, F> Collector for TakeWhile<C, F>
where
    C: Collector,
    F: FnMut(&C::Item) -> bool,
{
    type Item = C::Item;

    type Output = C::Output;

    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        let Some(ref mut pred) = self.pred else {
            return ControlFlow::Break(());
        };

        if pred(&item) {
            self.collector.collect(item)
        } else {
            self.pred = None;
            ControlFlow::Break(())
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.collector.finish()
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        let Some(ref mut pred) = self.pred else {
            return ControlFlow::Break(());
        };

        // Be careful - the underlying collector may stop before the predicate return false.
        let mut all_true = true;
        let cf = self.collector.collect_many(items.into_iter().take_while({
            // We wanna move `&mut pred`.
            // If we don't do this (use non-move closure directly),
            // `&mut pred` will be captured as `&mut &mut pred`.
            let all_true = &mut all_true;
            move |item| {
                // We trust the implementation of the standard library and the collector.
                // They should short-circuit on the first false.
                *all_true = pred(item);
                *all_true
            }
        }));

        if all_true {
            cf
        } else {
            self.pred = None;
            ControlFlow::Break(())
        }
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        if let Some(pred) = self.pred {
            self.collector
                .collect_then_finish(items.into_iter().take_while(pred))
        } else {
            self.collector.finish()
        }
    }
}

impl<C, F> RefCollector for TakeWhile<C, F>
where
    C: RefCollector,
    F: FnMut(&C::Item) -> bool,
{
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        let Some(ref mut pred) = self.pred else {
            return ControlFlow::Break(());
        };

        if pred(item) {
            self.collector.collect_ref(item)
        } else {
            self.pred = None;
            ControlFlow::Break(())
        }
    }
}

impl<C: Debug, F> Debug for TakeWhile<C, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TakeWhile")
            .field("collector", &self.collector)
            .finish()
    }
}
