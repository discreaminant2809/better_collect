pub mod cmp;
mod count;
mod fold;
mod fold_ref;
mod last;
pub mod num;
#[cfg(feature = "alloc")]
mod vec;
// mod sink;

pub use count::*;
pub use fold::*;
pub use fold_ref::*;
pub use last::*;
// pub use sink::*;
