use std::ops::ControlFlow;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;

#[cfg(feature = "alloc")]
use crate::{Collector, RefCollector};

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Collector for String {
    type Item = char;
    type Output = Self;

    #[inline]
    fn collect(&mut self, ch: char) -> ControlFlow<()> {
        self.push(ch);
        ControlFlow::Continue(())
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self
    }

    #[inline]
    fn collect_many(&mut self, items: impl IntoIterator<Item = char>) -> ControlFlow<()> {
        self.extend(items);
        ControlFlow::Continue(())
    }

    #[inline]
    fn collect_then_finish(mut self, items: impl IntoIterator<Item = char>) -> Self::Output {
        self.extend(items);
        self
    }
}

// #[cfg(feature = "alloc")]
// #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
// impl<'a> Collector<&'a str> for String {
//     type Output = Self;

//     #[inline]
//     fn collect(&mut self, s: &'a str) -> ControlFlow<()> {
//         self.push_str(s);
//         ControlFlow::Continue(())
//     }

//     #[inline]
//     fn finish(self) -> Self::Output {
//         self
//     }

//     #[inline]
//     fn collect_many(&mut self, items: impl IntoIterator<Item = &'a str>) -> ControlFlow<()> {
//         self.extend(items);
//         ControlFlow::Continue(())
//     }

//     #[inline]
//     fn collect_then_finish(mut self, items: impl IntoIterator<Item = &'a str>) -> Self::Output {
//         self.extend(items);
//         self
//     }
// }

// #[cfg(feature = "alloc")]
// #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
// impl Collector<String> for String {
//     type Output = Self;

//     #[inline]
//     fn collect(&mut self, s: String) -> ControlFlow<()> {
//         self.push_str(&s);
//         ControlFlow::Continue(())
//     }

//     #[inline]
//     fn finish(self) -> Self::Output {
//         self
//     }

//     #[inline]
//     fn collect_many(&mut self, items: impl IntoIterator<Item = String>) -> ControlFlow<()> {
//         self.extend(items);
//         ControlFlow::Continue(())
//     }

//     #[inline]
//     fn collect_then_finish(mut self, items: impl IntoIterator<Item = String>) -> Self::Output {
//         self.extend(items);
//         self
//     }
// }

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl RefCollector for String {
    #[inline]
    fn collect_ref(&mut self, &mut ch: &mut char) -> ControlFlow<()> {
        self.collect(ch)
    }
}

// #[cfg(feature = "alloc")]
// #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
// impl RefCollector<String> for String {
//     #[inline]
//     fn collect_ref(&mut self, item: &mut String) -> ControlFlow<()> {
//         <Self as Collector<&str>>::collect(self, item)
//     }
// }
