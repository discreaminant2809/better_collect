//! String-related [`Collector`]s.
//!
//! This module provides [`Collector`] implementations for [`String`] as well as
//! collectors for string concatenation.
//!
//! Collectors from [`String`] can collect `char`s. If you want to concat strings instead,
//! use [`ConcatStr`] or [`ConcatString`].
//!
//! This module corresponds to [`std::string`].

mod concat_str;
mod concat_string;

pub use concat_str::*;
pub use concat_string::*;

use std::ops::ControlFlow;

#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::{
    assert_ref_collector,
    collector::{Collector, RefCollector},
};

/// A [`RefCollector`] that pushes `char`s into a [`String`].
/// Its [`Output`] is [`String`].
///
/// This struct is created by `String::into_collector()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
pub struct IntoCollector(String);

/// A [`RefCollector`] that pushes `char`s into a [`&mut String`](String).
/// Its [`Output`] is [`&mut String`](String).
///
/// This struct is created by `String::collector_mut()`.
///
/// [`Collector`]: crate::collector::Collector
/// [`Output`]: crate::collector::Collector::Output
pub struct CollectorMut<'a>(&'a mut String);

impl crate::collector::IntoCollector for String {
    type Item = char;

    type Output = Self;

    type IntoCollector = IntoCollector;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        assert_ref_collector(IntoCollector(self))
    }
}

impl<'a> crate::collector::IntoCollector for &'a mut String {
    type Item = char;

    type Output = Self;

    type IntoCollector = CollectorMut<'a>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        assert_ref_collector(CollectorMut(self))
    }
}

impl Collector for IntoCollector {
    type Item = char;
    type Output = String;

    #[inline]
    fn collect(&mut self, ch: char) -> ControlFlow<()> {
        self.0.push(ch);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = char>) -> ControlFlow<()> {
        self.0.extend(items);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = char>) -> Self::Output {
        self.0.extend(items);
        self.0
    }
}

impl RefCollector for IntoCollector {
    #[inline]
    fn collect_ref(&mut self, &mut ch: &mut char) -> ControlFlow<()> {
        self.collect(ch)
    }
}

impl<'a> Collector for CollectorMut<'a> {
    type Item = char;
    type Output = &'a mut String;

    #[inline]
    fn collect(&mut self, ch: char) -> ControlFlow<()> {
        self.0.push(ch);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = char>) -> ControlFlow<()> {
        self.0.extend(items);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(self, items: impl IntoIterator<Item = char>) -> Self::Output {
        self.0.extend(items);
        self.0
    }
}

impl RefCollector for CollectorMut<'_> {
    #[inline]
    fn collect_ref(&mut self, &mut ch: &mut char) -> ControlFlow<()> {
        self.collect(ch)
    }
}
