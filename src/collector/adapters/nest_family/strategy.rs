use std::fmt::{Debug, DebugStruct};

use crate::collector::Collector;

pub trait Strategy {
    type Collector: Collector;

    fn next_collector(&mut self) -> Self::Collector;

    /// Needed because `CloneStrategy` and `FnMut` closure
    /// debug differently.
    fn debug(&self, debug_struct: &mut DebugStruct<'_, '_>)
    where
        Self::Collector: Debug,
    {
        let _ = debug_struct;
    }
}

#[derive(Clone)]
pub struct CloneStrategy<C>(C);

impl<C> CloneStrategy<C> {
    #[inline]
    pub fn new(x: C) -> Self {
        Self(x)
    }
}

impl<C> Strategy for CloneStrategy<C>
where
    C: Collector + Clone,
{
    type Collector = C;

    #[inline]
    fn next_collector(&mut self) -> Self::Collector {
        self.0.clone()
    }

    #[inline]
    fn debug(&self, debug_struct: &mut DebugStruct<'_, '_>)
    where
        Self::Collector: Debug,
    {
        debug_struct.field("inner_cloner", &self.0);
    }
}

impl<C, F> Strategy for F
where
    C: Collector,
    F: FnMut() -> C,
{
    type Collector = C;

    #[inline]
    fn next_collector(&mut self) -> Self::Collector {
        self()
    }
}
