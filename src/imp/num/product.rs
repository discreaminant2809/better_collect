use crate::{Collector, RefCollector};
use std::ops::ControlFlow;

/// A [`Collector`] that calculates sum of collected primitive numeric types.
///
/// This is a more specific version of [`Product`](crate::Product). This one needs less generics.
#[derive(Debug, Clone)]
pub struct Product<Num> {
    product: Num,
}

macro_rules! num_impl {
    ($num_ty:ty, $default:expr) => {
        impl Product<$num_ty> {
            /// Create a new instance of this collector with the initial value being
            /// the *additive identity* (“zero”) of the type.
            #[inline]
            pub const fn new() -> Self {
                Self { product: $default }
            }
        }

        // Roll out our own implementation since we can't use
        // 0.0 as the default value for f32 and f64
        // (their additive identity is -0.0, but the default value is 0.0)
        impl Default for Product<$num_ty> {
            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }

        impl Collector for Product<$num_ty> {
            type Item = $num_ty;
            type Output = $num_ty;

            #[inline]
            fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
                self.product += item;
                ControlFlow::Continue(())
            }

            #[inline]
            fn finish(self) -> Self::Output {
                self.product
            }

            /// Forwards to [`Iterator::sum`].
            #[inline]
            fn collect_many(
                &mut self,
                items: impl IntoIterator<Item = Self::Item>,
            ) -> ControlFlow<()> {
                self.product += items.into_iter().sum::<$num_ty>();
                ControlFlow::Continue(())
            }

            /// Forwards to [`Iterator::sum`].
            #[inline]
            fn collect_then_finish(
                self,
                items: impl IntoIterator<Item = Self::Item>,
            ) -> Self::Output {
                self.product + items.into_iter().sum::<$num_ty>()
            }
        }

        impl RefCollector for Product<$num_ty> {
            #[inline]
            fn collect_ref(&mut self, &mut item: &mut Self::Item) -> ControlFlow<()> {
                self.product += item;
                ControlFlow::Continue(())
            }
        }
    };
}

macro_rules! int_impls {
    ($($int_ty:ty)*) => {
        $(num_impl!($int_ty, 1);)*
    };
}

macro_rules! float_impls {
    ($($float_ty:ty)*) => {
        $(num_impl!($float_ty, 1.0);)*
    };
}

int_impls!(
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
);

float_impls!(f32 f64);
