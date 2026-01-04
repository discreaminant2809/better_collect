use std::ops::ControlFlow;

use super::{Collector, Combine, Funnel, IntoCollector};

use crate::{assert_collector, assert_ref_collector};

/// A [`Collector`] that can also collect items by mutable reference.
///
/// This trait introduces one additional method, [`collect_ref`](RefCollector::collect_ref),
/// which takes a mutable reference to an item.
///
/// It exists primarily to support [`combine()`].
/// Since [`Collector`] consumes items by ownership, each item cannot normally be passed further.
/// A type implementing this trait essentially declares: “A view of an item is enough for me
/// to collect it. Feel free to keep using it elsewhere.”
/// This enables items to flow through multiple collectors while maintaining composability.
/// See [`combine()`] for a deeper explanation.
///
/// # Difference from [`Collector<Item = &mut T>`]
///
/// Although both can collect mutable references, [`Collector<Item = &mut T>`]
/// implies ownership of those references and their lifetimes.
/// As such, it cannot be safely fed with references to items that will later be consumed.
///
/// For example, imagine a `Vec<&mut T>` collector:
/// it would hold the references beyond a single iteration,
/// preventing the item from being passed to another collector.
/// [`RefCollector`], in contrast, borrows mutably just long enough to collect,
/// then immediately releases the borrow, enabling true chaining.
///
/// # Dyn Compatibility
///
/// This trait is *dyn-compatible*, meaning it can be used as a trait object.
/// You do not need to specify the [`Output`](crate::collector::Collector::Output) type;
/// providing the [`Item`] type is enough.
///
/// For example:
///
/// ```no_run
/// # use better_collect::prelude::*;
/// # fn foo(_:
/// &mut dyn RefCollector<Item = i32>
/// # ) {}
/// ```
///
/// With the same [`Item`] type, a `dyn RefCollector` can be upcast to
/// a `dyn Collector`.
///
/// ```no_run
/// use better_collect::prelude::*;
///
/// let ref_collector: &mut dyn RefCollector<Item = i32> = &mut vec![].into_collector();
/// let collector: &mut dyn Collector<Item = i32> = ref_collector; // upcast
/// ```
///
/// [`combine()`]: RefCollector::combine
/// [`Item`]: crate::collector::Collector::Item
pub trait RefCollector: Collector {
    /// Collects an item and returns a [`ControlFlow`] indicating whether
    /// the collector has stopped accumulating right after this operation.
    ///
    /// See [`Collector::collect()`] for requirements regarding the returned [`ControlFlow`].
    ///
    /// After implementing this method, [`Collector::collect()`] can generally be forwarded
    /// like this:
    ///
    /// ```no_run
    /// # use better_collect::prelude::*;
    /// # use std::ops::ControlFlow;
    /// # struct Foo;
    /// # impl Collector for Foo {
    /// # type Item = ();
    /// # type Output = ();
    /// fn collect(&mut self, mut item: Self::Item) -> ControlFlow<()> {
    ///     self.collect_ref(&mut item)
    /// }
    /// #     fn finish(self) -> Self::Output {}
    /// # }
    /// # impl RefCollector for Foo {
    /// #     fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
    /// #         ControlFlow::Continue(())
    /// #     }
    /// # }
    /// ```
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()>;

    /// Use [`combine()`](RefCollector::combine).
    #[inline]
    #[deprecated(since = "0.3.0", note = "Use `combine()`")]
    fn then<C>(self, other: C) -> Combine<Self, C::IntoCollector>
    where
        Self: Sized,
        C: IntoCollector<Item = Self::Item>,
    {
        self.combine(other)
    }

    /// The most important adaptor. The reason why this crate exists.
    ///
    /// Creates a [`Collector`] that lets both collectors collect the same item.
    /// For each item collected, the first collector collects the item by mutable reference,
    /// then the second one collects it by either mutable reference or ownership.
    /// Together, they form a pipeline where each collector processes the item in turn,
    /// and the final one consumes by ownership.
    ///
    /// If the second collector implements [`RefCollector`], this adaptor implements [`RefCollector`],
    /// allowing the chain to be extended further with additional `combine()` calls.
    /// Otherwise, it becomes the endpoint of the pipeline.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::{prelude::*, cmp::Max};
    ///
    /// let mut collector = vec![].into_collector().combine(Max::new());
    ///
    /// assert!(collector.collect(4).is_continue());
    /// assert!(collector.collect(2).is_continue());
    /// assert!(collector.collect(6).is_continue());
    /// assert!(collector.collect(3).is_continue());
    ///
    /// assert_eq!(collector.finish(), (vec![4, 2, 6, 3], Some(6)));
    /// ```
    ///
    /// Even if one collector stops, `combine()` continues as the other does.
    /// It only stops when both collectors stop.
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let mut collector = vec![].into_collector().take(3).combine(()); // `()` always stops collecting.
    ///
    /// assert!(collector.collect(()).is_continue());
    /// assert!(collector.collect(()).is_continue());
    /// // Since `.take(3)` only takes 3 items,
    /// // it hints a stop right after the 3rd item is collected.
    /// assert!(collector.collect(()).is_break());
    /// # // Internal assertion.
    /// # assert!(collector.collect(()).is_break());
    ///
    /// assert_eq!(collector.finish(), (vec![(); 3], ()));
    /// ```
    ///
    /// Collectors can be chained with `combine()` as many as you want,
    /// as long as every of them except the last implements [`RefCollector`].
    ///
    /// Here’s the solution to [LeetCode #1491] to demonstrate it:
    ///
    /// ```
    /// use better_collect::{
    ///     prelude::*,
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
    ///                     .copying()
    ///                     .combine(Max::new().copying())
    ///                     .combine(Count::new())
    ///                     .combine(Sum::<i32>::new())
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
    fn combine<C>(self, other: C) -> Combine<Self, C::IntoCollector>
    where
        Self: Sized,
        C: IntoCollector<Item = Self::Item>,
    {
        assert_collector(Combine::new(self, other.into_collector()))
    }

    /// Creates a [`RefCollector`] that maps a mutable reference to an item
    /// into another mutable reference.
    ///
    /// This is used when a [`combine`] chain expects to collect `T`,
    /// but you have a collector that collects `U`. In that case,
    /// you can use `funnel()` to transform `U` into `T` before passing it along.
    ///
    /// Unlike [`Collector::map()`] or [`Collector::map_ref()`], this adaptor works
    /// seamlessly with [`RefCollector`]s by forwarding items directly through
    /// the [`collect_ref`] method.
    /// This avoids cloning because the underlying collector does not need owndership of items.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_collect::prelude::*;
    ///
    /// let vecs = [
    ///     vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
    ///     vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
    ///     vec!["swordswoman".to_owned(), "singer".to_owned()],
    /// ];
    ///
    /// let (concat_firsts, _lens) = vecs
    ///     .into_iter()
    ///     .better_collect(
    ///         ConcatString::new()
    ///             // We only need a reference to a string to concatenate.
    ///             // `funnel` lets us avoid cloning by transforming &mut Vec<_> → &mut String.
    ///             // Otherwise, we have to clone with `map_ref`.
    ///             .funnel(|v: &mut Vec<_>| &mut v[0])
    ///             .combine(vec![].into_collector().map(|v: Vec<_>| v.len()))
    ///     );
    ///
    /// assert_eq!(concat_firsts, "a1swordswoman");
    /// ```
    ///
    /// [`collect_ref`]: RefCollector::collect_ref
    /// [`combine`]: RefCollector::combine
    #[inline]
    fn funnel<F, T>(self, func: F) -> Funnel<Self, T, F>
    where
        Self: Sized,
        F: FnMut(&mut T) -> &mut Self::Item,
    {
        assert_ref_collector(Funnel::new(self, func))
    }
}

/// A mutable reference to a collect produce nothing.
///
/// This is useful when you just want to feed items to a collector without
/// finishing it.
impl<C: RefCollector> RefCollector for &mut C {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        C::collect_ref(self, item)
    }
}

macro_rules! dyn_impl {
    ($($traits:ident)*) => {
        impl<'a, T> Collector for &mut (dyn RefCollector<Item = T> $(+ $traits)* + 'a) {
            type Item = T;

            type Output = ();

            #[inline]
            fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
                <dyn RefCollector<Item = T>>::collect(*self, item)
            }

            #[inline]
            fn finish(self) -> Self::Output {}

            // The default implementation are sufficient.
        }

        impl<'a, T> RefCollector for &mut (dyn RefCollector<Item = T> $(+ $traits)* + 'a) {
            #[inline]
            fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
                <dyn RefCollector<Item = T>>::collect_ref(*self, item)
            }
        }
    };
}

dyn_impl!();
dyn_impl!(Send);
dyn_impl!(Sync);
dyn_impl!(Send Sync);

fn _dyn_compatible<T>(_: &mut dyn RefCollector<Item = T>) {}

fn _upcastable_to_collector<T>(x: &mut dyn RefCollector<Item = T>) -> &mut dyn Collector<Item = T> {
    x
}
