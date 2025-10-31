use std::ops::ControlFlow;

use crate::{Collector, Funnel, Then, assert_collector, assert_ref_collector};

pub trait RefCollector: Collector {
    /// Returns a [`ControlFlow`] to command whether to stop the collection.
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()>;

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
    /// use better_collect::{
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
    /// assert_eq!(collector.finish(), (vec![4, 2, 6, 3], Some(6)));
    /// ```
    ///
    /// Even if one collector stops, `Then` still continues as the other continues.
    /// It only stops when both collectors stop.
    ///
    /// ```
    /// use better_collect::{
    ///     Collector, RefCollector, cmp::Max, num::Sum,
    /// };
    ///
    /// let mut collector = vec![].take(3).then(()); // `()` always stops collecting.
    ///
    /// assert!(collector.collect(()).is_continue());
    /// assert!(collector.collect(()).is_continue());
    /// // Since `vec![].take(3)` only takes 3 items,
    /// // it signals a stop right after the 3rd item is collected.
    /// assert!(collector.collect(()).is_break());
    ///
    /// assert_eq!(collector.finish(), (vec![(); 3], ()));
    /// ```
    ///
    /// Collectors can be `then`ed as many as they can, as long as every of them except the last
    /// implements [`RefCollector`].
    ///
    /// Here is the solution of [LeetCode #1491] to demonstrate it:
    ///
    /// ```
    /// use better_collect::{
    ///     BetterCollect, Collector, RefCollector,
    ///     cmp::{Min, Max}, num::Sum, Count,
    /// };
    ///
    /// # struct Solution;
    /// impl Solution {
    ///     pub fn average(salary: Vec<i32>) -> f64 {
    ///         let (((min, max), count), sum) = salary
    ///             .into_iter()
    ///             .better_collect(
    ///                 Min::new()
    ///                     .copied()
    ///                     .then(Max::new().copied())
    ///                     .then(Count::new())
    ///                     .then(Sum::<i32>::new())
    ///             );
    ///                 
    ///         let (min, max) = (min.unwrap(), max.unwrap());
    ///         (sum - max - min) as f64 / (count - 2) as f64
    ///     }
    /// }
    ///
    /// fn correct(actual: f64, expected: f64) -> bool {
    ///     const DELTA: f64 = 1E-5;
    ///     (actual - expected).abs() <= DELTA
    /// }
    ///
    /// assert!(correct(
    ///     Solution::average(vec![5, 3, 1, 2]), 2.5
    /// ));
    /// assert!(correct(
    ///     Solution::average(vec![1, 2, 4]), 2.0
    /// ));
    /// ```
    ///
    /// [LeetCode #1491]: https://leetcode.com/problems/average-salary-excluding-the-minimum-and-maximum-salary
    #[inline]
    fn then<C>(self, other: C) -> Then<Self, C>
    where
        C: Collector<Item = Self::Item>,
    {
        assert_collector(Then::new(self, other))
    }

    #[inline]
    fn funnel<F, T>(self, func: F) -> Funnel<Self, T, F>
    where
        F: FnMut(&mut T) -> &mut Self::Item,
    {
        assert_ref_collector(Funnel::new(self, func))
    }
}
