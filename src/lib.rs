#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(not(feature = "std"))]
extern crate core as std;

mod adaptors;
mod imp;
mod traits;

pub use adaptors::*;
pub use imp::*;
pub use traits::*;

#[inline(always)]
fn assert_collector<C: Collector>(collector: C) -> C {
    collector
}

#[inline(always)]
fn assert_collector_by_ref<C: RefCollector>(collector: C) -> C {
    collector
}

#[cfg(test)]
mod tests {
    use crate::{BetterCollect, Collector, RefCollector};

    #[test]
    fn then() {
        let arr = [1, 2, 3];
        let (arr1, arr2) = arr.into_iter().better_collect(vec![].then(vec![]));
        assert_eq!(arr1, arr);
        assert_eq!(arr2, arr);

        let arr = ["1", "2", "3"];
        let (arr1, arr2) = ["1", "2", "3"]
            .into_iter()
            .map(String::from)
            .better_collect(vec![].cloned().then(vec![]));
        assert_eq!(arr1, arr);
        assert_eq!(arr2, arr);
    }
}
