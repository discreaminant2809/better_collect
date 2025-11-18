//! This module corresponds to the [`std::sync::mpsc`] module.

use std::{
    ops::ControlFlow,
    sync::mpsc::{Sender, SyncSender},
};

use crate::Collector;

/// A [`Collector`] that sends items through a [`std::sync::mpsc::channel()`].
///
/// Its [`Output`](crate::Collector::Output) is the original [`Sender`] that was
/// converted into this collector via [`into_collector()`](crate::IntoCollector::into_collector),
/// allowing you to retrieve it back.
///
/// Unlike [`send`](Sender::send), items collected after the
/// receiver has hung up are simply lost. They cannot be recovered.
///
/// # Examples
///
/// ```
/// use std::{thread, sync::mpsc};
/// use better_collect::{Collector, IntoCollector};
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

/// A [`Collector`] that sends items through a [`std::sync::mpsc::sync_channel()`].
///
/// Its [`Output`](crate::Collector::Output) is the original [`SyncSender`] that was
/// converted into this collector via [`into_collector()`](crate::IntoCollector::into_collector),
/// allowing you to retrieve it back.
///
/// Unlike [`send`](SyncSender::send()), items collected after the
/// receiver has hung up are simply lost. They cannot be recovered.
///
/// # Examples
///
/// ```
/// use std::{thread, sync::mpsc};
/// use better_collect::{Collector, IntoCollector};
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

impl<T> crate::IntoCollector for Sender<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = IntoCollector<T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        IntoCollector { sender: self }
    }
}

impl<T> Collector for IntoCollector<T> {
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

impl<T> crate::IntoCollector for SyncSender<T> {
    type Item = T;

    type Output = Self;

    type IntoCollector = IntoSyncCollector<T>;

    #[inline]
    fn into_collector(self) -> Self::IntoCollector {
        IntoSyncCollector { sender: self }
    }
}

impl<T> Collector for IntoSyncCollector<T> {
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
