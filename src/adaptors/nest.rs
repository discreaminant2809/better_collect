use std::ops::ControlFlow;

use crate::{Collector, Fuse, RefCollector};

/// A [`Collector`] that collects all outputs produced by an inner collector.
///
/// This `struct` is created by [`Collector::nest()`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Nest<CO, CI> {
    // It's possible that the active inner has been accumulating,
    // but then `break_hint` is called and it signals a stop.
    outer: Fuse<CO>,
    inner: CI,
    // An `Option` is neccessary here because we need to
    // track whether this active one has collected anything.
    // If it's just `CI`, `finish` can't know whether it's
    // an "empty" collector or not.
    active_inner: Option<CI>,
}

impl<CO, CI> Nest<CO, CI>
where
    CO: Collector<Item = CI::Output>,
    CI: Collector + Clone,
{
    pub(crate) fn new(outer: CO, inner: CI) -> Self {
        Self {
            outer: outer.fuse(),
            inner,
            active_inner: None,
        }
    }
}

impl<CO, CI> Collector for Nest<CO, CI>
where
    CO: Collector<Item = CI::Output>,
    CI: Collector + Clone,
{
    type Item = CI::Item;

    type Output = CO::Output;

    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        if self.outer.break_hint() {
            return ControlFlow::Break(());
        }

        let active_inner = if let Some(active_inner) = &mut self.active_inner {
            active_inner
        } else {
            loop {
                let active_inner = self.inner.clone();
                if !active_inner.break_hint() {
                    break self.active_inner.insert(active_inner);
                }

                self.outer.collect(active_inner.finish())?;
            }
        };

        if active_inner.collect(item).is_break() {
            self.outer.collect(
                self.active_inner
                    .take()
                    .expect("active_inner_collector should exist")
                    .finish(),
            )
        } else {
            ControlFlow::Continue(())
        }
    }

    fn finish(mut self) -> Self::Output {
        if let Some(active_inner_collector) = self.active_inner {
            let _ = self.outer.collect(active_inner_collector.finish());
        }

        self.outer.finish()
    }

    #[inline]
    fn break_hint(&self) -> bool {
        self.outer.break_hint()
    }

    // TODO: we should be clear about whether to check the outer's `break_hint`
    // repeatedly, or only once per inner rotation.
    // So that we can override `collect_many` and `collect_then_finish`.
}

impl<CO, CI> RefCollector for Nest<CO, CI>
where
    CO: Collector<Item = CI::Output>,
    CI: RefCollector + Clone,
{
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        if self.outer.break_hint() {
            return ControlFlow::Break(());
        }

        let active_inner = if let Some(active_inner) = &mut self.active_inner {
            active_inner
        } else {
            loop {
                let active_inner = self.inner.clone();
                if !active_inner.break_hint() {
                    break self.active_inner.insert(active_inner);
                }

                self.outer.collect(active_inner.finish())?;
            }
        };

        if active_inner.collect_ref(item).is_break() {
            self.outer.collect(
                self.active_inner
                    .take()
                    .expect("active_inner_collector should exist")
                    .finish(),
            )
        } else {
            ControlFlow::Continue(())
        }
    }
}
