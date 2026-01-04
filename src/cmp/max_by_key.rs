use std::{fmt::Debug, ops::ControlFlow};

use crate::{assert_collector, collector::Collector};

use super::{Max, value_key::ValueKey};

/// A [`Collector`] that computes the item among the items it collects
/// that gives the maximum value from a function.
///
/// Its [`Output`](Collector::Output) is `None` if it has not collected any items,
/// or `Some` containing the maximum item otherwise.
///
/// This collector is constructed by [`Max::by_key()`](super::Max::by_key).
///
/// This collector corresponds to [`Iterator::max_by_key()`].
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, cmp::Max};
///
/// let mut collector = Max::by_key(|s: &&str| s.len());
///
/// assert!(collector.collect("a").is_continue());
/// assert!(collector.collect("the").is_continue());
/// assert!(collector.collect("is").is_continue());
/// assert!(collector.collect("among").is_continue());
/// assert!(collector.collect("not").is_continue());
///
/// assert_eq!(collector.finish(), Some("among"));
/// ```
///
/// The output is `None` if no items were collected.
///
/// ```
/// use better_collect::{prelude::*, cmp::Max};
///
/// assert_eq!(Max::by_key(|s: &&str| s.len()).finish(), None);
/// ```
#[derive(Clone)]
pub struct MaxByKey<T, K, F> {
    value_key_collector: Max<ValueKey<T, K>>,
    f: F,
}

impl<T, K, F> MaxByKey<T, K, F>
where
    K: Ord,
    F: FnMut(&T) -> K,
{
    /// Creates a new instance of this collector with a given key-extraction function.
    #[deprecated(since = "0.3.0", note = "Use `Max::by_key`")]
    #[inline]
    pub const fn new(f: F) -> Self {
        assert_collector(Self {
            value_key_collector: Max::new(),
            f,
        })
    }
}

impl<T, K, F> Collector for MaxByKey<T, K, F>
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

impl<T: Debug, K: Debug, F> Debug for MaxByKey<T, K, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaxByKey")
            .field("max_value_key", &self.value_key_collector.max)
            .finish()
    }
}
