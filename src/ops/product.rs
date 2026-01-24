use crate::collector::CollectorBase;

///
pub trait Muling {
    ///
    type Output;

    ///
    type Muling: CollectorBase<Output = Self::Output>;

    ///
    fn muling() -> Self::Muling;
}
