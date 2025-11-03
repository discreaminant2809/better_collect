pub mod cmp;
#[cfg(feature = "alloc")]
mod collections;
mod count;
mod last;
pub mod num;
#[cfg(feature = "alloc")]
pub mod string;
mod try_fold;
mod try_fold_ref;
mod unit;
#[cfg(feature = "alloc")]
mod vec;
// mod sink;

pub use count::*;
pub use last::*;
pub use try_fold::*;
pub use try_fold_ref::*;
// pub use sink::*;
