use std::{fmt::Debug, ops::ControlFlow};

use crate::{Collector, assert_collector};

use super::{Min, value_key::ValueKey};

/// A [`Collector`] that computes the item among the items it collects
/// that gives the minimum value from a key-extraction function.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the minimum item otherwise.
///
/// This collector corresponds to [`Iterator::min_by_key()`].
///
/// # Examples
///
/// ```
/// use better_collect::{Collector, cmp::MinByKey};
///
/// let mut collector = MinByKey::new(|s: &&str| s.len());
///
/// assert!(collector.collect("force").is_continue());
/// assert!(collector.collect("the").is_continue());
/// assert!(collector.collect("is").is_continue());
/// assert!(collector.collect("among").is_continue());
/// assert!(collector.collect("not").is_continue());
///
/// assert_eq!(collector.finish(), Some("is"));
/// ```
///
/// The output is `None` if no items were collected.
///
/// ```
/// use better_collect::{Collector, cmp::MinByKey};
///
/// assert_eq!(MinByKey::new(|s: &&str| s.len()).finish(), None);
/// ```
#[derive(Clone)]
pub struct MinByKey<T, K, F> {
    value_key_collector: Min<ValueKey<T, K>>,
    f: F,
}

impl<T, K, F> MinByKey<T, K, F>
where
    K: Ord,
    F: FnMut(&T) -> K,
{
    /// Creates a new instance of this collector with a given key-extraction function.
    #[inline]
    pub const fn new(f: F) -> Self {
        assert_collector(Self {
            value_key_collector: Min::new(),
            f,
        })
    }
}

impl<T, K, F> Collector for MinByKey<T, K, F>
where
    K: Ord,
    F: FnMut(&T) -> K,
{
    type Item = T;

    type Output = Option<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        let item_value_key = ValueKey::new(item, &mut self.f);
        self.value_key_collector.collect(item_value_key)
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.value_key_collector.finish().map(ValueKey::into_value)
    }

    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.value_key_collector.collect_many(
            items
                .into_iter()
                .map(|item| ValueKey::new(item, &mut self.f)),
        )
    }

    fn collect_then_finish(self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        let Self {
            value_key_collector,
            mut f,
        } = self;

        value_key_collector
            .collect_then_finish(
                items
                    .into_iter()
                    .map(move |item| ValueKey::new(item, &mut f)),
            )
            .map(ValueKey::into_value)
    }
}

impl<T: Debug, K: Debug, F> Debug for MinByKey<T, K, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinByKey")
            .field("min_value_key", &self.value_key_collector.min)
            .finish()
    }
}
