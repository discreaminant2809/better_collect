use crate::Collector;

pub trait BetterCollect: Iterator {
    #[inline]
    fn better_collect<C>(&mut self, collector: C) -> C::Output
    where
        C: Collector<Item = Self::Item>,
    {
        collector.collect_then_finish(self)
    }
}

impl<I: Iterator> BetterCollect for I {}
