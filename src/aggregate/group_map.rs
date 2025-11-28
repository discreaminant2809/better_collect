use crate::{
    aggregate::{AggregateOp, Group, IntoAggregate, OccupiedGroup, VacantGroup},
    assert_collector,
};

/// A group map.
pub trait GroupMap {
    /// The key of each group.
    type Key;

    /// The value of each group.
    type Value;

    /// An existing group.
    type Occupied<'a>: OccupiedGroup<Key = Self::Key, Value = Self::Value>
    where
        Self: 'a;

    /// A group not existing yet.
    type Vacant<'a>: VacantGroup<Key = Self::Key, Value = Self::Value>
    where
        Self: 'a;

    /// Returns a [`Group`] for the given `key`, representing either an
    /// existing group or a new group that can be created.
    fn group<'a>(&'a mut self, key: Self::Key) -> Group<Self::Occupied<'a>, Self::Vacant<'a>>;

    /// Creates a [`Collector`] that aggregates items into groups
    ///
    /// This collects `(K, V)`s. Items that have the same key `K` go to the same group, and the way
    /// all values `V` of the same key are grouped is determined by the provided `op`.
    ///
    /// # Examples
    ///
    /// [`Collector`]: crate::Collector
    fn into_aggregate<Op>(self, op: Op) -> IntoAggregate<Self, Op>
    where
        Self: Sized,
        Op: AggregateOp<Key = Self::Key, Value = Self::Value>,
    {
        assert_collector(IntoAggregate::new(self, op))
    }
}
