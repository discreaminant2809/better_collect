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
    // An `Option` is neccessary here because this case exists:
    // if we use a bare `CI`, we create this collector, but then
    // we call `finish()` right away, it is incorrect for the outer
    // to collect this.
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
                    .expect("active_inner collector should exist")
                    .finish(),
            )
        } else {
            ControlFlow::Continue(())
        }
    }

    fn finish(mut self) -> Self::Output {
        if let Some(active_inner) = self.active_inner {
            // Due to this line, the outer has to be fused.
            let _ = self.outer.collect(active_inner.finish());
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

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        // Fuse is needed because in the previous cycle the inner may've exhausted the iterator.
        // If we peek again, we may accidentally unroll the iterator.
        // Could we use our own flag? Nope:
        // - If we implement our own iterator, we lose benefits from heavy specialization from Rust.
        // - If we use `inspect()`, we actually don't need to set the flag a lot and
        //   and the type of the iterator may not be preserved,
        //   hence again we lose benefits from heavy specialization.
        // In a nutshell, try to use what the standard library provides as much as possible.
        //
        // "B-but `peek()` pulls out one item prematurely." Not an issue!
        // If there is at least one item (hence that item is pulled out prematurely),
        // it is eventually collected by the inner anyway.
        // Why? In the loop that refreshing the inner, the `break` condition is that
        // the inner can still accumulate, which means we always have something
        // to collect that prematurely pulled item.
        let mut items = items.into_iter().fuse().peekable();

        // Are there still more items? If no, don't put me in a loop!
        while items.peek().is_some() {
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

            if active_inner.collect_many(&mut items).is_break() {
                self.outer.collect(
                    self.active_inner
                        .take()
                        .expect("active_inner collector should exist")
                        .finish(),
                )?;
            } else {
                // the inner still returns `Continue(())`. The iterator is definitely exhausted...
                break;
            }

            // ...but if it doesn't, we can't conclude whether it's exhausted or not.
            // It is possible that the inner stops AND the iterator is exhausted or not.
            // That's why we need the `fuse().peekable()` combo. It guards both cases.
        }

        ControlFlow::Continue(())
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        let mut items = items.into_iter().fuse().peekable();

        // Clear the remaining active inner first if any.
        if let Some(active_inner) = self.active_inner
            && self
                .outer
                .collect(active_inner.collect_then_finish(&mut items))
                .is_break()
        {
            return self.outer.finish();
        }

        todo!("the active_inner should use collect_then_finish()")
    }
}

impl<CO, CI> RefCollector for Nest<CO, CI>
where
    CO: Collector<Item = CI::Output>,
    CI: RefCollector + Clone,
{
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
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
                    .expect("active_inner collector should exist")
                    .finish(),
            )
        } else {
            ControlFlow::Continue(())
        }
    }
}
