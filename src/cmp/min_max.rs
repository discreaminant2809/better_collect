use std::ops::ControlFlow;

use itertools::MinMaxResult;

use crate::collector::{Collector, CollectorBase};

use super::{max_assign, min_assign};

/// A collector that computes the minimum and maximum values among the items it collects.
///
/// Its [`Output`](CollectorBase::Output) is:
///
/// - [`MinMaxResult::NoElements`] if no items were collected.
/// - [`MinMaxResult::OneElement`] containing one item if exactly that item was collected.
/// - [`MinMaxResult::MinMax`] containing the minimum and the maximum items (in order)
///   if two or more items were collected.
///
///   If there are multiple equally minimum items, the first one collected is returned.
///   If there are multiple equally maximum items, the last one collected is returned.
///
/// This collector corresponds to [`Itertools::minmax()`](itertools::Itertools::minmax).
///
/// # Examples
///
/// ```
/// use better_collect::{prelude::*, cmp::MinMax};
/// use itertools::MinMaxResult;
///
/// assert_eq!(
///     [].into_iter().feed_into(MinMax::<i32>::new()),
///     MinMaxResult::NoElements,
/// );
/// assert_eq!(
///     [1].into_iter().feed_into(MinMax::new()),
///     MinMaxResult::OneElement(1),
/// );
/// assert_eq!(
///     [1, 3, 2].into_iter().feed_into(MinMax::new()),
///     MinMaxResult::MinMax(1, 3),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct MinMax<T> {
    state: State<T>,
}

#[derive(Debug, Clone)]
enum State<T> {
    NoElements,
    OneElement(T),
    MinMax { min: T, max: T, prev: Option<T> },
}

impl<T> MinMax<T>
where
    T: Ord,
{
    /// Creates a new instance of this collector.
    #[inline]
    pub const fn new() -> Self {
        Self {
            state: State::NoElements,
        }
    }
}

impl<T> CollectorBase for MinMax<T>
where
    T: Ord,
{
    type Output = MinMaxResult<T>;

    fn finish(self) -> Self::Output {
        match self.state {
            State::NoElements => MinMaxResult::NoElements,
            State::OneElement(item) => MinMaxResult::OneElement(item),
            State::MinMax {
                min,
                max,
                prev: Some(prev),
            } if prev < min => MinMaxResult::MinMax(prev, max),
            State::MinMax {
                min,
                max,
                prev: Some(prev),
            } if max <= prev => MinMaxResult::MinMax(min, prev),
            State::MinMax { min, max, .. } => MinMaxResult::MinMax(min, max),
        }
    }
}

impl<T> Collector<T> for MinMax<T>
where
    T: Ord,
{
    #[inline]
    fn collect(&mut self, item: T) -> ControlFlow<()> {
        match &mut self.state {
            State::NoElements => self.state = State::OneElement(item),
            State::OneElement(_) => {
                let State::OneElement(prev) = std::mem::take(self).state else {
                    unreachable!("the state is somehow incorrect");
                };

                if item < prev {
                    self.state = State::MinMax {
                        min: item,
                        max: prev,
                        prev: None,
                    }
                } else {
                    self.state = State::MinMax {
                        min: prev,
                        max: item,
                        prev: None,
                    }
                }
            }
            State::MinMax { min, max, prev } => {
                let Some(prev) = prev.take() else {
                    *prev = Some(item);
                    return ControlFlow::Continue(());
                };

                if item < prev {
                    min_assign(min, item);
                    max_assign(max, prev);
                } else {
                    min_assign(min, prev);
                    max_assign(max, item);
                }
            }
        }

        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = T>) -> ControlFlow<()> {
        let mut items = items.into_iter();

        'outer: loop {
            match &mut self.state {
                State::NoElements => {
                    self.state = {
                        let Some(item) = items.next() else {
                            break;
                        };
                        State::OneElement(item)
                    }
                }
                State::OneElement(_) => {
                    let Some(item) = items.next() else {
                        break;
                    };

                    let State::OneElement(prev) = std::mem::take(self).state else {
                        unreachable!("the state is somehow incorrect");
                    };

                    if item < prev {
                        self.state = State::MinMax {
                            min: item,
                            max: prev,
                            prev: None,
                        }
                    } else {
                        self.state = State::MinMax {
                            min: prev,
                            max: item,
                            prev: None,
                        }
                    }
                }
                State::MinMax { min, max, prev } => {
                    let Some(mut first) = prev.take().or_else(|| items.next()) else {
                        break;
                    };

                    let Some(mut second) = items.next() else {
                        *prev = Some(first);
                        break;
                    };

                    loop {
                        if second < first {
                            min_assign(min, second);
                            max_assign(max, first);
                        } else {
                            min_assign(min, first);
                            max_assign(max, second);
                        }

                        match items.next() {
                            Some(item) => first = item,
                            None => break 'outer,
                        }

                        match items.next() {
                            Some(item) => second = item,
                            None => {
                                *prev = Some(first);
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn collect_then_finish(mut self, items: impl IntoIterator<Item = T>) -> Self::Output {
        let mut items = items.into_iter();

        'outer: loop {
            match self.state {
                State::NoElements => {
                    self.state = {
                        let Some(item) = items.next() else {
                            break MinMaxResult::NoElements;
                        };
                        State::OneElement(item)
                    }
                }
                State::OneElement(prev) => {
                    let Some(item) = items.next() else {
                        break MinMaxResult::OneElement(prev);
                    };

                    if item < prev {
                        self.state = State::MinMax {
                            min: item,
                            max: prev,
                            prev: None,
                        }
                    } else {
                        self.state = State::MinMax {
                            min: prev,
                            max: item,
                            prev: None,
                        }
                    }
                }
                State::MinMax {
                    mut min,
                    mut max,
                    prev,
                } => {
                    let Some(mut first) = prev.or_else(|| items.next()) else {
                        break MinMaxResult::MinMax(min, max);
                    };

                    let Some(mut second) = items.next() else {
                        break if first < min {
                            MinMaxResult::MinMax(first, max)
                        } else if max <= first {
                            MinMaxResult::MinMax(min, first)
                        } else {
                            MinMaxResult::MinMax(min, max)
                        };
                    };

                    loop {
                        if second < first {
                            min_assign(&mut min, second);
                            max_assign(&mut max, first);
                        } else {
                            min_assign(&mut min, first);
                            max_assign(&mut max, second);
                        }

                        match items.next() {
                            Some(item) => first = item,
                            None => break 'outer MinMaxResult::MinMax(min, max),
                        }

                        match items.next() {
                            Some(item) => second = item,
                            None => {
                                break 'outer if first < min {
                                    MinMaxResult::MinMax(first, max)
                                } else if max <= first {
                                    MinMaxResult::MinMax(min, first)
                                } else {
                                    MinMaxResult::MinMax(min, max)
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<T> Default for MinMax<T>
where
    T: Ord,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "std"))]
mod proptests {
    use itertools::Itertools;

    use proptest::collection::vec as propvec;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

    use crate::test_utils::{BasicCollectorTester, CollectorTesterExt, PredError};

    use super::super::test_utils::Id;
    use super::*;

    proptest! {
        #[test]
        fn all_collect_methods(
            nums in propvec(any::<i32>(), ..=3),
            starting_nums in propvec(any::<i32>(), ..=3),
        ) {
            all_collect_methods_impl(nums, starting_nums)?;
        }
    }

    fn all_collect_methods_impl(nums: Vec<i32>, starting_nums: Vec<i32>) -> TestCaseResult {
        BasicCollectorTester {
            iter_factory: || nums.iter().enumerate().map(|(id, &num)| Id { id, num }),
            collector_factory: || {
                let mut collector = MinMax::new();
                let _ = collector.collect_many(
                    starting_nums
                        .iter()
                        .zip(nums.len()..)
                        .map(|(&num, id)| Id { id, num }),
                );
                collector
            },
            should_break_pred: |_| false,
            pred: |iter, output, remaining| {
                let iter = starting_nums
                    .iter()
                    .zip(nums.len()..)
                    .map(|(&num, id)| Id { id, num })
                    .chain(iter);

                if !Id::full_eq_minmax_res(iter.minmax(), output) {
                    Err(PredError::IncorrectOutput)
                } else if remaining.next().is_some() {
                    Err(PredError::IncorrectIterConsumption)
                } else {
                    Ok(())
                }
            },
        }
        .test_collector()
    }
}
