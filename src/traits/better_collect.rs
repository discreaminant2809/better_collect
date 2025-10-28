use crate::Collector;

pub trait BetterCollect: Iterator {
    #[inline]
    fn better_collect<C: Collector<Self::Item>>(&mut self, collector: C) -> C::Output {
        collector.collect_then_finish(self)
    }
}

impl<I: Iterator> BetterCollect for I {}
