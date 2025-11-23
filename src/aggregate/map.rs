use crate::{
    aggregate::{AggregateOp, Entry, OccupiedEntry, VacantEntry, into_aggregate::IntoAggregate},
    assert_collector,
};

///
pub trait Map {
    type Key;

    type Value;

    type Occupied<'a>: OccupiedEntry<Key = Self::Key, Value = Self::Value>
    where
        Self: 'a;

    type Vacant<'a>: VacantEntry<Key = Self::Key, Value = Self::Value>
    where
        Self: 'a;

    fn entry<'a>(&'a mut self, key: Self::Key) -> Entry<Self::Occupied<'a>, Self::Vacant<'a>>;

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

    use crate::aggregate::{self, Map};

    impl<'a, K, V> aggregate::VacantEntry for VacantEntry<'a, K, V> {
        type Key = K;

        type Value = V;

        fn key(&self) -> &Self::Key {
            self.key()
        }

        fn insert(self, value: Self::Value) {
            self.insert(value);
        }
    }

    impl<'a, K, V> aggregate::OccupiedEntry for OccupiedEntry<'a, K, V> {
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

    impl<K: Eq + Hash, V> Map for HashMap<K, V> {
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

        fn entry<'a>(
            &'a mut self,
            key: Self::Key,
        ) -> aggregate::Entry<Self::Occupied<'a>, Self::Vacant<'a>> {
            match self.entry(key) {
                Entry::Occupied(entry) => aggregate::Entry::Occupied(entry),
                Entry::Vacant(entry) => aggregate::Entry::Vacant(entry),
            }
        }
    }
}
