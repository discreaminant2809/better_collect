use std::marker::PhantomData;

use crate::aggregate::AggregateOp;

pub struct Combine<V, F, G, Ops> {
    new_fn: F,
    modify_fn: G,
    ops: Ops,
    _marker: PhantomData<fn(&mut V) -> V>,
}

impl<V, F, G, Ops> Combine<V, F, G, Ops>
where
    Ops: Tuple<V, F, G>,
    F: FnMut(&Ops::Key, Ops::Item) -> V,
    G: FnMut(&mut V, Ops::TeeModifier<'_>),
{
    pub fn new(new_fn: F, modify_fn: G, ops: Ops) -> Self {
        Self {
            new_fn,
            modify_fn,
            ops,
            _marker: PhantomData,
        }
    }
}

impl<V, F, G, Ops> AggregateOp for Combine<V, F, G, Ops>
where
    Ops: Tuple<V, F, G>,
    F: FnMut(&Ops::Key, Ops::Item) -> V,
    G: FnMut(&mut V, Ops::TeeModifier<'_>),
{
    type Key = Ops::Key;

    type Value = V;

    type Item = Ops::Item;

    #[inline]
    fn new_value(&mut self, key: &Self::Key, item: Self::Item) -> Self::Value {
        (self.new_fn)(key, item)
    }

    #[inline]
    fn modify(&mut self, value: &mut Self::Value, item: Self::Item) {
        (self.modify_fn)(value, self.ops.tee_modifier(item));
    }
}

pub struct CombineModifier<'a, Op>
where
    Op: AggregateOp,
{
    op: &'a mut Op,
    item: Op::Item,
}

impl<Op> CombineModifier<'_, Op>
where
    Op: AggregateOp,
{
    #[inline]
    pub fn modify(self, value: &mut Op::Value) {
        self.op.modify(value, self.item);
    }
}

trait Sealed {}

#[doc(hidden)] // Needed (plus the trait being `pub`) due to E0446.
#[allow(private_bounds)]
pub trait Tuple<V, F, G>: Sealed + Sized {
    type Key;

    type Item;

    type TeeModifier<'a>
    where
        Self: 'a;

    fn tee_modifier(&mut self, item: Self::Item) -> Self::TeeModifier<'_>;
}

impl<K, Op0, Op1> Sealed for (Op0, Op1)
where
    Op0: AggregateOp<Key = K>,
    Op1: AggregateOp<Key = K>,
{
}

impl<Op0, Op1, K, V, F, G> Tuple<V, F, G> for (Op0, Op1)
where
    Op0: AggregateOp<Key = K>,
    Op1: AggregateOp<Key = K>,
{
    type Key = K;

    type Item = (Op0::Item, Op1::Item);

    type TeeModifier<'a>
        = (CombineModifier<'a, Op0>, CombineModifier<'a, Op1>)
    where
        Self: 'a;

    #[inline]
    fn tee_modifier(&mut self, item: Self::Item) -> Self::TeeModifier<'_> {
        (
            CombineModifier {
                op: &mut self.0,
                item: item.0,
            },
            CombineModifier {
                op: &mut self.1,
                item: item.1,
            },
        )
    }
}
