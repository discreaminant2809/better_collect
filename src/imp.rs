mod all_any;
pub mod cmp;
#[cfg(feature = "alloc")]
mod collections;
mod count;
mod find;
mod last;
pub mod num;
mod reduce;
#[cfg(feature = "alloc")]
pub mod string;
mod sum;
mod try_fold;
mod try_fold_ref;
mod unit;
#[cfg(feature = "alloc")]
mod vec;
// mod sink;

pub use all_any::*;
pub use count::*;
pub use find::*;
pub use last::*;
pub use reduce::*;
pub use sum::*;
pub use try_fold::*;
pub use try_fold_ref::*;
// pub use sink::*;
