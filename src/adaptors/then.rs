use std::ops::ControlFlow;

use super::Fuse;
use crate::{Collector, RefCollector};

/// A [`Collector`] that lets **both** collectors collect the same item.
///
/// This `struct` is created by [`RefCollector::then()`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Then<C1, C2> {
    collector1: Fuse<C1>,
    collector2: C2,
}

impl<C1, C2> Then<C1, C2> {
    pub(crate) fn new(collector1: C1, collector2: C2) -> Self {
        Self {
            collector1: Fuse::new(collector1),
            collector2,
        }
    }
}

impl<C1, C2> Collector for Then<C1, C2>
where
    C1: RefCollector,
    C2: Collector<Item = C1::Item>,
{
    type Item = C1::Item;
    type Output = (C1::Output, C2::Output);

    #[inline]
    fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
        match (
            self.collector1.collect_ref(&mut item),
            self.collector2.collect(item),
        ) {
            (ControlFlow::Break(_), ControlFlow::Break(_)) => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        (self.collector1.finish(), self.collector2.finish())
    }

    // fn reserve(&mut self, additional_min: usize, additional_max: Option<usize>) {
    //     let (lower1, upper1) = self.collector1.size_hint();

    //     // Both have the same theme: the 2nd collector reserves the left-over amount.
    //     let (reserve_lower1, reserve_lower2) = if additional_min > lower1 {
    //         (lower1, additional_min - lower1)
    //     } else {
    //         (additional_min, 0)
    //     };

    //     let (reserve_upper1, reserve_upper2) = match (additional_max, upper1) {
    //         (Some(additional_max), Some(upper1)) if additional_max > upper1 => {
    //             (Some(upper1), Some(additional_max - upper1))
    //         }
    //         (additional_max, _) => (additional_max, Some(0)),
    //     };

    //     self.collector1.reserve(reserve_lower1, reserve_upper1);
    //     self.collector2.reserve(reserve_lower2, reserve_upper2);
    // }

    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     let (lower1, upper1) = self.collector1.size_hint();
    //     let (lower2, upper2) = self.collector2.size_hint();

    //     (
    //         lower1.saturating_add(lower2),
    //         (|| upper1?.checked_add(upper2?))(),
    //     )
    // }

    // fn inactivity_hint(&self) -> Option<usize> {
    //     match (
    //         self.collector1.inactivity_hint(),
    //         self.collector2.inactivity_hint(),
    //     ) {
    //         (Some(count1), Some(count2)) => Some(count1.min(count2)),
    //         (Some(count), None) | (None, Some(count)) => Some(count),
    //         (None, None) => None,
    //     }
    // }

    // fn skip_till_active(&mut self, max: Option<usize>) {
    //     match (
    //         self.collector1.inactivity_hint(),
    //         self.collector2.inactivity_hint(),
    //     ) {
    //         (Some(count1), Some(count2)) => {
    //             let max = match max {
    //                 Some(max) => max.min(count1.min(count2)),
    //                 None => count1.min(count2),
    //             };

    //             self.collector1.skip_till_active(Some(max));
    //             self.collector2.skip_till_active(Some(max));
    //         }
    //         (Some(_), None) => {
    //             self.collector1.skip_till_active(max);
    //         }
    //         (None, Some(_)) => {
    //             self.collector2.skip_till_active(max);
    //         }
    //         (None, None) => {}
    //     }
    // }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        // DO NOT do this. `Iterator::next` is inefficient for repeated calls if the iterator
        // is "segmented," like `chain`, `skip`, etc.
        // while let Some(mut item) = items.next() {
        //     if self.collector1.collect_ref(&mut item).is_break() {
        //         return self.collector2.collect_many(items);
        //     }
        //     if self.collector2.collect(item).is_break() {
        //         return self.collector1.collect_many(items);
        //     }
        // }

        // Do this instead: forward to `try_for_each` since it's overriden to be more efficient.
        match items.try_for_each(|mut item| {
            if self.collector1.collect_ref(&mut item).is_break() {
                // Save the item for the 2nd collector, or else the item will be lost,
                // and the second collector is not be able to collect it.
                ControlFlow::Break(Which::First(item))
            } else if self.collector2.collect(item).is_break() {
                // The 1st collector has collected this item, so we don't need to save.
                ControlFlow::Break(Which::Second)
            } else {
                ControlFlow::Continue(())
            }
        }) {
            ControlFlow::Continue(_) => ControlFlow::Continue(()),
            ControlFlow::Break(Which::First(item)) => self
                .collector2
                .collect_many(Some(item).into_iter().chain(items)),
            ControlFlow::Break(Which::Second) => self.collector1.collect_many(items),
        }

        // loop {
        //     // We should, try at best, not consume any item just to end up that the collector accepts no more items.
        //     if self.inactivity_hint().is_none() {
        //         break ControlFlow::Break(());
        //     }

        //     // Use `tro_for_each` since it's often overriden to be more efficient (e.g. `chain`, `skip`, etc.)
        //     match items.try_for_each(|mut item| {
        //         match self.collector1.inactivity_hint() {
        //             Some(0) => {
        //                 if self.collector1.collect_ref(&mut item).is_break() {
        //                     return ControlFlow::Break(WhichInactive::First {
        //                         item,
        //                         inactive_for: None,
        //                     });
        //                 }
        //             }
        //             inactive_for => {
        //                 // We have to save the item before exiting.
        //                 return ControlFlow::Break(WhichInactive::First { item, inactive_for });
        //             }
        //         }

        //         match self.collector2.inactivity_hint() {
        //             Some(0) => debug_assert!(self.collector2.collect(item).is_continue()),
        //             inactive_for => {
        //                 // But we don't need here, since the above has already conllected it.
        //                 return ControlFlow::Break(WhichInactive::Second { inactive_for });
        //             }
        //         }

        //         ControlFlow::Continue(())
        //     }) {
        //         ControlFlow::Continue(_) => break ControlFlow::Continue(()),

        //         ControlFlow::Break(WhichInactive::First {
        //             item,
        //             inactive_for: Some(count),
        //         }) => {
        //             self.collector2
        //                 .collect_many(Some(item).into_iter().chain(items.by_ref()).take(count))?;
        //         }
        //         ControlFlow::Break(WhichInactive::First { item, .. }) => {
        //             break self
        //                 .collector2
        //                 .collect_many(Some(item).into_iter().chain(items));
        //         }

        //         ControlFlow::Break(WhichInactive::Second {
        //             inactive_for: Some(count),
        //         }) => {
        //             self.collector1.collect_many(items.by_ref().take(count))?;
        //         }
        //         ControlFlow::Break(WhichInactive::Second { .. }) => {
        //             break self.collector1.collect_many(items);
        //         }
        //     }
        // }
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        let mut items = items.into_iter();

        match items.try_for_each(|mut item| {
            if self.collector1.collect_ref(&mut item).is_break() {
                // Save the item for the 2nd collector, or else the item will be lost,
                // violating the semantics.
                ControlFlow::Break(Which::First(item))
            } else if self.collector2.collect(item).is_break() {
                // The 1st collector has collected this item, so we don't need to save.
                ControlFlow::Break(Which::Second)
            } else {
                ControlFlow::Continue(())
            }
        }) {
            ControlFlow::Continue(_) => self.finish(),
            ControlFlow::Break(Which::First(item)) => (
                self.collector1.finish(),
                self.collector2
                    .collect_then_finish(Some(item).into_iter().chain(items)),
            ),
            ControlFlow::Break(Which::Second) => (
                self.collector1.collect_then_finish(items),
                self.collector2.finish(),
            ),
        }
    }
}

impl<C1, C2> RefCollector for Then<C1, C2>
where
    C1: RefCollector,
    C2: RefCollector<Item = C1::Item>,
{
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        match (
            self.collector1.collect_ref(item),
            self.collector2.collect_ref(item),
        ) {
            (ControlFlow::Break(_), ControlFlow::Break(_)) => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }
}

// A helper enum for `collect_many` and `collect_then_finish` to know which has finished.
enum Which<T> {
    First(T),
    Second,
}
