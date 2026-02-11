//! [`Collector`]s for comparing items.
//!
//! This module provides collectors that determine the maximum or minimum
//! values among the items they collect, using different comparison strategies.
//! They correspond to [`Iterator`]’s comparison-related methods, such as
//! [`Iterator::max()`], [`Iterator::min_by()`], and [`Iterator::max_by_key()`].
//!
//! This module corresponds to [`std::cmp`].
//!
//! [`Collector`]: crate::collector::Collector

mod max;
mod max_by;
mod max_by_key;
mod min;
mod min_by;
mod min_by_key;
mod value_key;
// mod is_sorted;
// mod is_sorted_by;
// mod is_sorted_by_key;

pub use max::*;
pub use max_by::*;
pub use max_by_key::*;
pub use min::*;
pub use min_by::*;
pub use min_by_key::*;

#[cfg(test)]
#[allow(dead_code)]
mod test_utils {
    use std::cmp::Ordering;

    /// A struct that never compares the ID.
    /// This is crucial to test that the correct item is pertained
    /// if there are multiple equal maximal/minimal items.
    #[derive(Debug, Clone, Copy, Eq)]
    pub struct Id {
        pub id: usize,
        pub num: i32,
    }

    impl Id {
        pub fn full_eq(self, other: Self) -> bool {
            self.id == other.id && self.num == other.num
        }

        pub fn full_eq_opt(x: Option<Self>, y: Option<Self>) -> bool {
            match (x, y) {
                (Some(x), Some(y)) => x.full_eq(y),
                (None, None) => true,
                _ => false,
            }
        }
    }

    impl PartialEq for Id {
        fn eq(&self, other: &Self) -> bool {
            self.num == other.num
        }
    }

    impl PartialOrd for Id {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Id {
        fn cmp(&self, other: &Self) -> Ordering {
            self.num.cmp(&other.num)
        }
    }
}
