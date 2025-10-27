use crate::{Collector, RefCollector};
use std::ops::ControlFlow;

#[derive(Debug, Default)]
pub struct Sum<Num> {
    accum: Num,
}

macro_rules! num_impl {
    ($num_ty:ty) => {
        impl Sum<$num_ty> {
            #[inline]
            pub const fn new() -> Self {
                Self { accum: 0 as _ }
            }
        }

        impl Collector for Sum<$num_ty> {
            type Item = $num_ty;

            type Output = $num_ty;

            #[inline]
            fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
                self.accum += item;
                ControlFlow::Continue(())
            }

            #[inline]
            fn finish(self) -> Self::Output {
                self.accum
            }

            /// Forwards to [`Iterator::sum`].
            #[inline]
            fn collect_many(
                &mut self,
                items: impl IntoIterator<Item = Self::Item>,
            ) -> ControlFlow<()> {
                self.accum += items.into_iter().sum::<$num_ty>();
                ControlFlow::Continue(())
            }

            /// Forwards to [`Iterator::sum`].
            #[inline]
            fn collect_then_finish(
                self,
                items: impl IntoIterator<Item = Self::Item>,
            ) -> Self::Output {
                self.accum + items.into_iter().sum::<$num_ty>()
            }
        }

        impl RefCollector for Sum<$num_ty> {
            #[inline]
            fn collect_ref(&mut self, &mut item: &mut Self::Item) -> ControlFlow<()> {
                self.accum += item;
                ControlFlow::Continue(())
            }
        }
    };
}

macro_rules! num_impls {
    ($($num_ty:ty)*) => {
        $(num_impl!($num_ty);)*
    };
}

num_impls!(
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
    f32 f64
);
