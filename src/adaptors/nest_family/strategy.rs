use crate::Collector;

pub trait Strategy {
    type Collector: Collector;

    fn next_collector(&mut self) -> Self::Collector;
}

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
