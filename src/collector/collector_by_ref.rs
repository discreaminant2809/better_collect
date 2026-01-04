use super::{Collector, IntoCollector};

/// A type that can be converted into a collector by shared reference.
///
/// This trait's main purpose is to provide a convenience method to creates
/// a collector from `&T`.
///
/// You cannot implement this trait directly.
/// Instead, you should implement [`IntoCollector`] for `&T` (where `T` is your type)
/// and this trait is automatically implemented for `T`.
///
/// This trait is not intended for use in bounds.
/// Use [`IntoCollector`] in trait bounds instead.
#[allow(private_bounds)]
pub trait CollectorByRef: Sealed {
    /// Which collector being produced?
    type Collector<'a>: Collector
    where
        Self: 'a;

    /// Creates a [`Collector`] from a shared reference of a value.
    fn collector(&self) -> Self::Collector<'_>;
}

impl<T> CollectorByRef for T
where
    for<'a> &'a T: IntoCollector,
{
    type Collector<'a>
        = <&'a T as IntoCollector>::IntoCollector
    where
        T: 'a;

    #[inline]
    fn collector(&self) -> Self::Collector<'_> {
        self.into_collector()
    }
}

trait Sealed {}

impl<T> Sealed for T where for<'a> &'a T: IntoCollector {}
