use std::{fmt::Debug, ops::ControlFlow};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;

use crate::{Collector, RefCollector};

/// A [`RefCollector`] that concatenates [`String`] into a single [`String`].
///
/// Its [`Output`](Collector::Output) type is [`String`].
///
/// # Examples
///
/// ```
/// use better_collect::{BetterCollect, string::ConcatString};
///
/// let url = ["https://", "website.com", "/login"]
///     .into_iter()
///     .map(String::from)
///     .better_collect(ConcatString::new());
///
/// assert_eq!(url, "https://website.com/login");
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Debug, Default, Clone)]
pub struct ConcatString {
    buf: String,
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl ConcatString {
    /// Creates a new instance of this collector with an empty string.
    pub const fn new() -> Self {
        Self { buf: String::new() }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
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

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl RefCollector for ConcatString {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.buf.push_str(item);
        ControlFlow::Continue(())
    }
}
