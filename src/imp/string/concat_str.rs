use std::{fmt::Debug, marker::PhantomData, ops::ControlFlow};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;

use crate::{Collector, RefCollector};

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Clone, Default)]
pub struct ConcatStr<'a> {
    buf: String,
    _marker: PhantomData<fn(&'a str)>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl ConcatStr<'_> {
    pub const fn new() -> Self {
        Self {
            buf: String::new(),
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> Collector for ConcatStr<'a> {
    type Item = &'a str;

    type Output = String;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        self.buf.push_str(item);
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
impl RefCollector for ConcatStr<'_> {
    #[inline]
    fn collect_ref(&mut self, item: &mut Self::Item) -> ControlFlow<()> {
        self.buf.push_str(item);
        ControlFlow::Continue(())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Debug for ConcatStr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcatStr").field("buf", &self.buf).finish()
    }
}
