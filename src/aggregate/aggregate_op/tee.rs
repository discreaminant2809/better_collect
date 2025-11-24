use std::marker::PhantomData;

use crate::aggregate::{AggregateOp, RefAggregateOp, assert_op};

///
pub struct Combine<V, F, G, Ops> {
    new_fn: F,
    get_mut_fn: G,
    ops: Ops,
    _marker: PhantomData<fn(&mut V) -> V>,
}

impl<V, F, G, Ops> Combine<V, F, G, Ops>
where
    Ops: Tuple<V, F, G>,
    F: FnMut(&Ops::Key, Ops::Values) -> V,
    G: FnMut(&mut V) -> Ops::ValuesMut<'_>,
{
    /// Creates a new instance of this aggregate op.
    pub fn new(new_fn: F, get_mut_fn: G, ops: Ops) -> Self {
        assert_op(Self {
            new_fn,
            get_mut_fn,
            ops,
            _marker: PhantomData,
        })
    }
}

impl<V, F, G, Ops> AggregateOp for Combine<V, F, G, Ops>
where
    Ops: Tuple<V, F, G>,
    F: FnMut(&Ops::Key, Ops::Values) -> V,
    G: FnMut(&mut V) -> Ops::ValuesMut<'_>,
{
    type Key = Ops::Key;

    type Value = V;

    type Item = Ops::Item;

    #[inline]
    fn new_value(&mut self, key: &Self::Key, item: Self::Item) -> Self::Value {
        let values = self.ops.new_value(key, item);
        (self.new_fn)(key, values)
    }

    #[inline]
    fn modify(&mut self, value: &mut Self::Value, item: Self::Item) {
        let values = (self.get_mut_fn)(value);
        self.ops.modify(values, item);
    }
}

trait Sealed {}

#[doc(hidden)] // Needed (plus the trait being `pub`) due to E0446.
#[allow(private_bounds)]
pub trait Tuple<V, F, G>: Sealed + Sized {
    type Key;

    type Item;

    type Values;

    type ValuesMut<'a>
    where
        Self: 'a;

    fn new_value(&mut self, key: &Self::Key, item: Self::Item) -> Self::Values;

    fn modify<'a>(&mut self, values: Self::ValuesMut<'a>, item: Self::Item)
    where
        Self: 'a;
}

impl<K, Op0, Op1> Sealed for (Op0, Op1)
where
    Op0: RefAggregateOp<Key = K>,
    Op1: AggregateOp<Key = K>,
{
}

impl<Op0, Op1, K, It, V, F, G> Tuple<V, F, G> for (Op0, Op1)
where
    Op0: RefAggregateOp<Key = K, Item = It>,
    Op1: AggregateOp<Key = K, Item = It>,
{
    type Key = K;

    type Item = It;

    type Values = (Op0::Value, Op1::Value);

    type ValuesMut<'a>
        = (&'a mut Op0::Value, &'a mut Op1::Value)
    where
        Self: 'a;

    fn new_value(&mut self, key: &Self::Key, mut item: Self::Item) -> Self::Values {
        let (op0, op1) = self;
        (op0.new_value_ref(key, &mut item), op1.new_value(key, item))
    }

    fn modify<'a>(&mut self, values: Self::ValuesMut<'a>, mut item: Self::Item)
    where
        Self: 'a,
    {
        let (op0, op1) = self;
        let (value0, value1) = values;

        op0.modify_ref(value0, &mut item);
        op1.modify(value1, item);
    }
}
