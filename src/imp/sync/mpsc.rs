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
/// If the receiver has hung up, this collector returns [`Break(())`](ControlFlow::Break).
///
/// Unlike [`send`](Sender::send), items collected after the
/// receiver has hung up are simply lost. They cannot be recovered.
///
/// This struct is created by `Sender::into_collector()`.
///
/// # Examples
///
/// ```
/// use std::{thread, sync::{mpsc, Mutex, Condvar}};
/// use better_collect::prelude::*;
///
/// let (tx, rx) = mpsc::channel();
/// let hung = Mutex::new(false);
/// let notifier = Condvar::new();
///
/// thread::scope(|s| {
///     let handle = s.spawn(|| {
///         let mut tx = tx.into_collector();
///
///         assert!(tx.collect_many([1, 2, 3]).is_continue());
///
///         // Wait until the receiver hangs.
///         notifier.wait_while(
///             hung.lock().unwrap(),
///             |hung| !*hung,
///         );
///
///         assert!(tx.collect(4).is_break());
///     });
///
///     assert_eq!(rx.recv(), Ok(1));
///     assert_eq!(rx.recv(), Ok(2));
///     assert_eq!(rx.recv(), Ok(3));
///     
///     drop(rx);
///     *hung.lock().unwrap() = true;
///     notifier.notify_one();
///     assert!(handle.join().is_ok());
/// });
/// ```
pub struct IntoCollector<T> {
    sender: Sender<T>,
}

/// A [`Collector`] that sends items through a [`std::sync::mpsc::channel()`].
/// Its [`Output`](crate::Collector::Output) is [`&Sender`](Sender).
///
/// If the receiver has hung up, this collector returns [`Break(())`](ControlFlow::Break).
///
/// Unlike [`send`](Sender::send), items collected after the
/// receiver has hung up are simply lost. They cannot be recovered.
///
/// This struct is created by `Sender::collector()`.
///
/// # Examples
///
/// ```
/// use std::{thread, sync::{mpsc, Mutex, Condvar}};
/// use better_collect::prelude::*;
///
/// let (tx, rx) = mpsc::channel();
/// let hung = Mutex::new(false);
/// let notifier = Condvar::new();
///
/// thread::scope(|s| {
///     let handle = s.spawn(|| {
///         let mut tx = tx.collector();
///
///         assert!(tx.collect_many([1, 2, 3]).is_continue());
///
///         // Wait until the receiver hangs.
///         notifier.wait_while(
///             hung.lock().unwrap(),
///             |hung| !*hung,
///         );
///
///         assert!(tx.collect(4).is_break());
///     });
///
///     assert_eq!(rx.recv(), Ok(1));
///     assert_eq!(rx.recv(), Ok(2));
///     assert_eq!(rx.recv(), Ok(3));
///     
///     drop(rx);
///     *hung.lock().unwrap() = true;
///     notifier.notify_one();
///     assert!(handle.join().is_ok());
/// });
/// ```
pub struct Collector<'a, T>(&'a Sender<T>);

/// A [`Collector`] that sends items through a [`std::sync::mpsc::sync_channel()`].
/// Its [`Output`](crate::Collector::Output) is [`SyncSender`].
///
/// If the receiver has hung up, this collector returns [`Break(())`](ControlFlow::Break).
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
///
/// let handle = thread::spawn(move || {
///     let mut tx = tx.into_collector();
///
///     assert!(tx.collect_many([1, 2, 3]).is_continue());
///     assert!(tx.collect(4).is_break());
/// });
///
/// assert_eq!(rx.recv(), Ok(1));
/// assert_eq!(rx.recv(), Ok(2));
/// assert_eq!(rx.recv(), Ok(3));
///
/// drop(rx);
/// assert!(handle.join().is_ok());
/// ```
pub struct IntoSyncCollector<T> {
    sender: SyncSender<T>,
}

/// A [`Collector`] that sends items through a [`std::sync::mpsc::sync_channel()`].
/// Its [`Output`](crate::Collector::Output) is [`&SyncSender`](SyncSender).
///
/// If the receiver has hung up, this collector returns [`Break(())`](ControlFlow::Break).
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
/// let handle = thread::spawn(move || {
///     let mut tx = tx.collector();
///
///     assert!(tx.collect_many([1, 2, 3]).is_continue());
///     assert!(tx.collect(4).is_break());
/// });
///
/// assert_eq!(rx.recv(), Ok(1));
/// assert_eq!(rx.recv(), Ok(2));
/// assert_eq!(rx.recv(), Ok(3));
///
/// drop(rx);
/// assert!(handle.join().is_ok());
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
