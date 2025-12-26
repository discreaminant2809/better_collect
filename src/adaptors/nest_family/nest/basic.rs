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
            // We collect in a loop. Should we use `collect_many()` for the outer?
            // Nah. The best I've tried is `iter::repeat_with(...).map_while(...)`.
            // This combo guarantees a `(0, None)` size hint, which has little
            // to no chance of optimize. Not to mention "fraudulent" `collect_many()`
            // implementation. The outer may return `Continue(())` even tho
            // it hasn't exhausted the iterator, leading to more guards
            // => goodbye optimization.
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

    // TODO: the overrides are still buggy

    // fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
    //     // Fuse is needed because in the previous cycle the inner may've exhausted the iterator.
    //     // If we peek again, we may accidentally unroll the iterator.
    //     // Could we use our own flag? Nope:
    //     // - If we implement our own iterator, we lose benefits from heavy specialization from Rust.
    //     // - If we use `inspect()`, we actually don't need to set the flag a lot and
    //     //   and the type of the iterator may not be preserved,
    //     //   hence again we lose benefits from heavy specialization.
    //     // In a nutshell, try to use what the standard library provides as much as possible.
    //     //
    //     // "B-but `peek()` pulls out one item prematurely." Not an issue!
    //     // If there is at least one item (hence that item is pulled out prematurely),
    //     // it is eventually collected by the inner anyway.
    //     // Why? In the loop that refreshing the inner, the `break` condition is that
    //     // the inner can still accumulate, which means we always have something
    //     // to collect that prematurely pulled item.
    //     let mut items = items.into_iter().fuse().peekable();

    //     self.outer.collect_many(std::iter::from_fn(|| {
    //         let active_inner = if let Some(active_inner) = &mut self.active_inner {
    //             active_inner
    //         } else {
    //             let active_inner = self.inner.clone();
    //             if active_inner.break_hint() {
    //                 return Some(active_inner.finish());
    //             }

    //             self.active_inner.insert(active_inner)
    //         };

    //         active_inner.collect_many(&mut items).is_break().then(|| {
    //             self.active_inner
    //                 .take()
    //                 .expect("active_inner collector should exist")
    //                 .finish()
    //         })
    //     }))

    //     // // Are there still more items? If no, don't put me in a loop!
    //     // while items.peek().is_some() {
    //     //     let active_inner = if let Some(active_inner) = &mut self.active_inner {
    //     //         active_inner
    //     //     } else {
    //     //         loop {
    //     //             let active_inner = self.inner.clone();
    //     //             if !active_inner.break_hint() {
    //     //                 break self.active_inner.insert(active_inner);
    //     //             }

    //     //             self.outer.collect(active_inner.finish())?;
    //     //         }
    //     //     };

    //     //     if active_inner.collect_many(&mut items).is_break() {
    //     //         self.outer.collect(
    //     //             self.active_inner
    //     //                 .take()
    //     //                 .expect("active_inner collector should exist")
    //     //                 .finish(),
    //     //         )?;
    //     //     } else {
    //     //         // the inner still returns `Continue(())`. The iterator is definitely exhausted...
    //     //         break;
    //     //     }

    //     //     // ...but if it doesn't, we can't conclude whether it's exhausted or not.
    //     //     // It is possible that the inner stops AND the iterator is exhausted or not.
    //     //     // That's why we need the `fuse().peekable()` combo. It guards both cases.
    //     // }

    //     // ControlFlow::Continue(())
    // }

    // fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
    //     let mut items = items.into_iter().fuse().peekable();

    //     // Both blocks look `chain`able, but unfortunably we can't share the `items`
    //     // and/or avoid premature clone.
    //     if let Some(active_inner) = self.active_inner
    //         && self
    //             .outer
    //             .collect(active_inner.collect_then_finish(&mut items))
    //             .is_break()
    //     {
    //         return self.outer.finish();
    //     }

    //     self.outer.collect_then_finish(std::iter::from_fn(move || {
    //         let active_inner = self.inner.clone();
    //         // To prevent pulling one item prematurely.
    //         if active_inner.break_hint() {
    //             return Some(active_inner.finish());
    //         }

    //         items.peek()?;
    //         Some(active_inner.collect_then_finish(&mut items))
    //     }))
    // }
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn doc_example() {
        let mut collector = vec![]
            .into_collector()
            .nest(vec![].into_collector().take(3));

        assert!(collector.collect_many(1..=9).is_continue());

        assert_eq!(collector.finish(), [[1, 2, 3], [4, 5, 6], [7, 8, 9],],);
    }
}
