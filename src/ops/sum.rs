use crate::collector::CollectorBase;

///
pub trait Adding {
    ///
    type Output;

    ///
    type Adding: CollectorBase<Output = Self::Output>;

    ///
    fn adding() -> Self::Adding;
}
