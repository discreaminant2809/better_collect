use std::ops::ControlFlow;

use crate::{Collector, Funnel, Then, assert_collector, assert_ref_collector};

pub trait RefCollector<T>: Collector<T> {
    /// Returns a [`ControlFlow`] to command whether to stop the collection.
    fn collect_ref(&mut self, item: &mut T) -> ControlFlow<()>;

    /// The most important adaptor, and the reason why this crate exists.
    ///
    /// `then()` returns a new collector that let both collectors receive the same item. The first collector
    /// receives a mutable reference to the item, **then** the second collector may receive it by
    /// mutable reference or ownership. Together, they form a "pipeline" that the item passes through collectors,
    /// and the end is the final consumer by ownership.
    ///
    /// If the second collector implements [`RefCollector`], this collector can be `then`ed further to extend the pipeline,
    /// else it cannot be `then`ed and becomes the endpoint of the pipeline.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collector::{
    ///     Collector, RefCollector, cmp::Max, num::Sum,
    /// };
    ///
    /// let mut collector = vec![].then(Max::new());
    ///
    /// assert!(collector.collect(4).is_continue());
    /// assert!(collector.collect(2).is_continue());
    /// assert!(collector.collect(6).is_continue());
    /// assert!(collector.collect(3).is_continue());
    ///
    /// assert_eq!(collector.finish(), ([4, 2, 6, 3], 6));
    /// ```
    ///
    /// Even if one collector stops, `Then` still continues as the other continues.
    /// It only stops when both collectors stop.
    ///
    /// ```
    /// use better_collector::{
    ///     Collector, RefCollector, cmp::Max, num::Sum,
    /// };
    ///
    /// let mut collector = vec![].take(3).then(()); // `()` always stops collecting.
    ///
    /// assert!(collector.collect(4).is_continue());
    /// assert!(collector.collect(2).is_continue());
    /// assert!(collector.collect(6).is_continue());
    ///
    /// assert_eq!(collector.clone().finish(), ([4, 2, 6], ()));
    ///
    /// assert!(collector.collect(1).is_break());
    /// ```
    #[inline]
    fn then<C>(self, other: C) -> Then<Self, C>
    where
        C: Collector<T>,
    {
        assert_collector(Then::new(self, other))
    }

    #[inline]
    fn funnel<F, U>(self, func: F) -> Funnel<Self, U, F>
    where
        F: FnMut(&mut U) -> &mut T,
    {
        assert_ref_collector(Funnel::new(self, func))
    }
}
