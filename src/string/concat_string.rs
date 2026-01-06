#![allow(deprecated)]

use std::{fmt::Debug, ops::ControlFlow};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;

use crate::collector::{Collector, RefCollector};

/// Use [`Concat`]'s methods.
///
/// [`Concat`]: crate::slice::Concat
#[deprecated(since = "0.4.0", note = "Use `Concat`'s methods")]
#[derive(Debug, Default, Clone)]
pub struct ConcatString {
    buf: String,
}

impl ConcatString {
    /// Creates a new instance of this collector with an empty string.
    #[inline]
    pub const fn new() -> Self {
        Self::with_buf(String::new())
    }

    /// Creates a new instance of this collector with an initial string.
    #[inline]
    pub const fn with_buf(buf: String) -> Self {
        Self { buf }
    }
}

impl Collector for ConcatString {
    type Item = String;

    type Output = String;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.buf.push_str(&item);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.buf
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = Self::Item>) -> ControlFlow<()> {
        self.buf.extend(items);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = Self::Item>) -> Self::Output {
        self.buf.extend(items);
        self.buf
    }
}

impl RefCollector for ConcatString {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.buf.push_str(item);
        ControlFlow::Continue(())
    }
}

impl From<String> for ConcatString {
    #[inline]
    fn from(buf: String) -> Self {
        Self::with_buf(buf)
    }
}
