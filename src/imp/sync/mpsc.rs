//! [`Collector`]s for [`Sender`] and [`SyncSender`].
//!
//! This module corresponds to [`std::sync::mpsc`].

use std::{
    ops::ControlFlow,
    sync::mpsc::{Sender, SyncSender},
};

/// A [`Collector`] that sends items through a [`std::sync::mpsc::channel()`].
/// Its [`Output`](crate::Collector::Output) is [`Sender`].
///
/// Unlike [`send`](Sender::send), items collected after the
/// receiver has hung up are simply lost. They cannot be recovered.
///
/// This struct is created by `Sender::into_collector()`.
///
/// # Examples
///
/// ```
/// use std::{thread, sync::mpsc};
/// use better_collect::prelude::*;
///
/// let (tx, rx) = mpsc::channel();
/// let mut tx = tx.into_collector();
///
/// thread::spawn(move || {
///     tx.collect_many([1, 2, 3]);
/// });
///
/// assert_eq!(rx.recv(), Ok(1));
/// assert_eq!(rx.recv(), Ok(2));
/// assert_eq!(rx.recv(), Ok(3));
/// assert!(rx.recv().is_err());
/// ```
pub struct IntoCollector<T> {
    sender: Sender<T>,
}

/// A [`Collector`] that sends items through a [`std::sync::mpsc::channel()`].
/// Its [`Output`](crate::Collector::Output) is [`&Sender`](Sender).
///
/// Unlike [`send`](Sender::send), items collected after the
/// receiver has hung up are simply lost. They cannot be recovered.
///
/// This struct is created by `Sender::collector()`.
///
/// # Examples
///
/// ```
/// use std::{thread, sync::mpsc};
/// use better_collect::prelude::*;
///
/// let (tx, rx) = mpsc::channel();
///
/// thread::spawn(move || {
///     tx.collector().collect_many([1, 2, 3]);
/// });
///
/// assert_eq!(rx.recv(), Ok(1));
/// assert_eq!(rx.recv(), Ok(2));
/// assert_eq!(rx.recv(), Ok(3));
/// assert!(rx.recv().is_err());
/// ```
pub struct Collector<'a, T>(&'a Sender<T>);

/// A [`Collector`] that sends items through a [`std::sync::mpsc::sync_channel()`].
/// Its [`Output`](crate::Collector::Output) is [`SyncSender`].
///
/// Unlike [`send`](SyncSender::send), items collected after the
/// receiver has hung up are simply lost. They cannot be recovered.
///
/// This struct is created by `SyncSender::into_collector()`.
///
/// # Examples
///
/// ```
/// use std::{thread, sync::mpsc};
/// use better_collect::prelude::*;
///
/// let (tx, rx) = mpsc::sync_channel(1);
/// let mut tx = tx.into_collector();
///
/// thread::spawn(move || {
///     tx.collect_many([1, 2, 3]);
/// });
///
/// assert_eq!(rx.recv(), Ok(1));
/// assert_eq!(rx.recv(), Ok(2));
/// assert_eq!(rx.recv(), Ok(3));
/// assert!(rx.recv().is_err());
/// ```
pub struct IntoSyncCollector<T> {
    sender: SyncSender<T>,
}

/// A [`Collector`] that sends items through a [`std::sync::mpsc::sync_channel()`].
/// Its [`Output`](crate::Collector::Output) is [`&SyncSender`](SyncSender).
///
/// Unlike [`send`](SyncSender::send), items collected after the
/// receiver has hung up are simply lost. They cannot be recovered.
///
/// This struct is created by `SyncSender::collector()`.
///
/// # Examples
///
/// ```
/// use std::{thread, sync::mpsc};
/// use better_collect::prelude::*;
///
/// let (tx, rx) = mpsc::sync_channel(1);
///
/// thread::spawn(move || {
///     tx.collector().collect_many([1, 2, 3]);
/// });
///
/// assert_eq!(rx.recv(), Ok(1));
/// assert_eq!(rx.recv(), Ok(2));
/// assert_eq!(rx.recv(), Ok(3));
/// assert!(rx.recv().is_err());
/// ```
pub struct SyncCollector<'a, T>(&'a SyncSender<T>);

impl<T> crate::IntoCollector for Sender<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = IntoCollector<T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        IntoCollector { sender: self }
    }
}

impl<T> crate::Collector for IntoCollector<T> {
    type Item = T;

    type Output = Sender<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        match self.sender.send(item) {
            Ok(_) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.sender
    }

    // The default implementations for other methods are sufficient.
}

impl<'a, T> crate::IntoCollector for &'a Sender<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = Collector<'a, T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        Collector(self)
    }
}

impl<'a, T> crate::Collector for Collector<'a, T> {
    type Item = T;

    type Output = &'a Sender<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        match self.0.send(item) {
            Ok(_) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0
    }

    // The default implementations for other methods are sufficient.
}

impl<T> crate::IntoCollector for SyncSender<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = IntoSyncCollector<T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        IntoSyncCollector { sender: self }
    }
}

impl<T> crate::Collector for IntoSyncCollector<T> {
    type Item = T;

    type Output = SyncSender<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        match self.sender.send(item) {
            Ok(_) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.sender
    }

    // The default implementations for other methods are sufficient.
}

impl<'a, T> crate::IntoCollector for &'a SyncSender<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = SyncCollector<'a, T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        SyncCollector(self)
    }
}

impl<'a, T> crate::Collector for SyncCollector<'a, T> {
    type Item = T;

    type Output = &'a SyncSender<T>;

    #[inline]
    fn collect(&mut self, item: Self::Item) -> ControlFlow<()> {
        match self.0.send(item) {
            Ok(_) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        }
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self.0
    }

    // The default implementations for other methods are sufficient.
}
