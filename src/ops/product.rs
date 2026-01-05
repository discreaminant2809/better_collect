use std::{fmt::Debug, iter, marker::PhantomData, ops::ControlFlow, ops::MulAssign};

use crate::{assert_collector, collector::Collector};

/// A [`Collector`] that computes the product of all collected items.
///
/// It is generic over two types:
///
/// - `A`: the accumulator and final result type.
///   Must implement both [`Product<T>`] and [`MulAssign<Self>`].
/// - `T`: the item type that this collector consumes.
///
/// This collector corresponds to [`Iterator::product()`], except that its return type
/// is slightly more restrictive (additionally requiring [`MulAssign<Self>`]).
///
/// Because this is an “umbrella” implementation which has more generics than needed
/// (cumbersome to initialize) and does **not** implement [`RefCollector`],
/// you can define your own `Product` collector tailored to your type, with fewer generics
/// and optional [`RefCollector`] implementation.
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, ops::Product};
///
/// let mut collector = Product::<i32, _>::new();
///
/// assert!(collector.collect(1).is_continue());
/// assert!(collector.collect(2).is_continue());
/// assert!(collector.collect(3).is_continue());
/// assert!(collector.collect(4).is_continue());
///
/// assert_eq!(collector.finish(), 24);
/// ```
///
/// [`RefCollector`]: crate::collector::RefCollector
pub struct Product<A, T> {
    product: A,
    _marker: PhantomData<fn(T)>,
}

impl<A: iter::Product<T> + MulAssign, T> Product<A, T> {
    /// Create a new instance of this collector with the initial value being
    /// the *additive identity* (“zero”) of the type.
    #[inline]
    pub fn new() -> Self {
        assert_collector(Self {
            product: None.into_iter().product(),
            _marker: PhantomData,
        })
    }
}

impl<A: iter::Product<T> + MulAssign, T> Collector for Product<A, T> {
    type Item = T;

    type Output = A;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.collect_many(Some(item))
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.product
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.product *= items.into_iter().product::<A>();
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.product *= items.into_iter().product::<A>();
        self.product
    }
}

impl<A: iter::Product<T> + MulAssign, T> Default for Product<A, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Clone, T> Clone for Product<A, T> {
    fn clone(&self) -> Self {
        Self {
            product: self.product.clone(),
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.product.clone_from(&source.product);
    }
}

impl<A: Debug, T> Debug for Product<A, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Product")
            .field("product", &self.product)
            .finish()
    }
}
