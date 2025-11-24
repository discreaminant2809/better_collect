use crate::{
    aggregate::{AggregateOp, Group, OccupiedGroup, VacantGroup, into_aggregate::IntoAggregate},
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

    ///
    fn into_aggregate<Op>(self, op: Op) -> IntoAggregate<Self, Op>
    where
        Self: Sized,
        Op: AggregateOp<Key = Self::Key, Value = Self::Value>,
    {
        assert_collector(IntoAggregate::new(self, op))
    }
}

// FIXME: move it to crate::imp::collections::hash_map.
#[cfg(feature = "std")]
mod hash_map {
    use std::collections::hash_map::*;
    use std::hash::Hash;

    use crate::aggregate::{self, GroupMap};

    impl<'a, K, V> aggregate::VacantGroup for VacantEntry<'a, K, V> {
        type Key = K;

        type Value = V;

        fn key(&self) -> &Self::Key {
            self.key()
        }

        fn insert(self, value: Self::Value) {
            self.insert(value);
        }
    }

    impl<'a, K, V> aggregate::OccupiedGroup for OccupiedEntry<'a, K, V> {
        type Key = K;

        type Value = V;

        fn key(&self) -> &Self::Key {
            self.key()
        }

        fn value(&self) -> &Self::Value {
            self.get()
        }

        fn value_mut(&mut self) -> &mut Self::Value {
            self.get_mut()
        }
    }

    impl<K: Eq + Hash, V> GroupMap for HashMap<K, V> {
        type Key = K;

        type Value = V;

        type Vacant<'a>
            = VacantEntry<'a, K, V>
        where
            Self: 'a;

        type Occupied<'a>
            = OccupiedEntry<'a, K, V>
        where
            Self: 'a;

        fn group<'a>(
            &'a mut self,
            key: Self::Key,
        ) -> aggregate::Group<Self::Occupied<'a>, Self::Vacant<'a>> {
            match self.entry(key) {
                Entry::Occupied(entry) => aggregate::Group::Occupied(entry),
                Entry::Vacant(entry) => aggregate::Group::Vacant(entry),
            }
        }
    }
}
