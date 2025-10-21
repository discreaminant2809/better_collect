use crate::Collector;

pub trait BetterCollect: Iterator {
    fn better_collect<C: Collector<Item = Self::Item>>(&mut self, mut collector: C) -> C::Output {
        // We don't care whether the collector breaks or not, since if it doesn't it'll have
        // completely depleted the iterator so... we just finish--nothing changed.
        let _ = collector.collect_many(self);
        collector.finish()
    }
}

impl<I: Iterator> BetterCollect for I {}
