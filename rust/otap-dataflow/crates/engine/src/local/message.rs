// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Abstraction to represent generic local senders and receivers.

use crate::channel_metrics::{
    ChannelMetricsHandle, ChannelMetricsRegistry, ChannelQueueDepth, ChannelReceiverMetricSets,
    ChannelReceiverMetricsState, ChannelSendErrorType, ChannelSenderMetricSets,
    ChannelSenderMetricsState, LocalChannelQueueDepth, LocalChannelReceiverMetricsHandle,
    LocalChannelSenderMetricsHandle,
};
use otap_df_channel::error::{RecvError, SendError};
use otap_df_channel::{mpmc, mpsc};
use otap_df_config::SignalType;
use std::cell::RefCell;
use std::rc::Rc;

enum LocalSenderInner<T> {
    Mpsc(mpsc::Sender<T>),
    Mpmc(mpmc::Sender<T>),
}

/// A generic local channel Sender.
#[must_use = "A `Sender` is requested but not used."]
pub struct LocalSender<T> {
    inner: LocalSenderInner<T>,
    metrics: Option<LocalChannelSenderMetricsHandle>,
    queue_depth: Option<LocalChannelQueueDepth>,
    signal: Option<fn(&T) -> Option<SignalType>>,
}

impl<T> Clone for LocalSender<T> {
    fn clone(&self) -> Self {
        let inner = match &self.inner {
            LocalSenderInner::Mpsc(sender) => LocalSenderInner::Mpsc(sender.clone()),
            LocalSenderInner::Mpmc(sender) => LocalSenderInner::Mpmc(sender.clone()),
        };
        Self {
            inner,
            metrics: self.metrics.clone(),
            queue_depth: self.queue_depth.clone(),
            signal: self.signal,
        }
    }
}

impl<T> LocalSender<T> {
    /// Creates a new local MPSC sender.
    pub const fn mpsc(sender: mpsc::Sender<T>) -> Self {
        Self {
            inner: LocalSenderInner::Mpsc(sender),
            metrics: None,
            queue_depth: None,
            signal: None,
        }
    }

    /// Creates a new local MPSC sender with metrics attached.
    pub(crate) fn mpsc_with_metrics(
        sender: mpsc::Sender<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: ChannelSenderMetricSets,
        queue_depth: LocalChannelQueueDepth,
        signal: Option<fn(&T) -> Option<SignalType>>,
    ) -> Self {
        let handle = Rc::new(RefCell::new(ChannelSenderMetricsState::new(metrics)));
        channel_metrics.register(ChannelMetricsHandle::LocalSender(handle.clone()));
        let mut sender = Self::mpsc(sender);
        sender.metrics = Some(handle);
        sender.queue_depth = Some(queue_depth);
        sender.signal = signal;
        sender
    }

    /// Creates a new local MPMC sender.
    pub const fn mpmc(sender: mpmc::Sender<T>) -> Self {
        Self {
            inner: LocalSenderInner::Mpmc(sender),
            metrics: None,
            queue_depth: None,
            signal: None,
        }
    }

    /// Creates a new local MPMC sender with metrics attached.
    pub(crate) fn mpmc_with_metrics(
        sender: mpmc::Sender<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: ChannelSenderMetricSets,
        queue_depth: LocalChannelQueueDepth,
        signal: Option<fn(&T) -> Option<SignalType>>,
    ) -> Self {
        let handle = Rc::new(RefCell::new(ChannelSenderMetricsState::new(metrics)));
        channel_metrics.register(ChannelMetricsHandle::LocalSender(handle.clone()));
        let mut sender = Self::mpmc(sender);
        sender.metrics = Some(handle);
        sender.queue_depth = Some(queue_depth);
        sender.signal = signal;
        sender
    }

    pub(crate) fn into_mpsc(self) -> Result<mpsc::Sender<T>, Self> {
        let LocalSender {
            inner,
            metrics,
            queue_depth,
            signal,
        } = self;
        match inner {
            LocalSenderInner::Mpsc(sender) => Ok(sender),
            LocalSenderInner::Mpmc(sender) => Err(Self {
                inner: LocalSenderInner::Mpmc(sender),
                metrics,
                queue_depth,
                signal,
            }),
        }
    }

    /// Sends a message to the channel.
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        let signal = self.signal.and_then(|extract| extract(&msg));
        let result = match &self.inner {
            LocalSenderInner::Mpsc(sender) => sender.send_async(msg).await,
            LocalSenderInner::Mpmc(sender) => sender.send_async(msg).await,
        };

        if result.is_ok()
            && let Some(queue_depth) = &self.queue_depth
        {
            queue_depth.record_send();
        }
        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.try_borrow_mut() {
                match &result {
                    Ok(()) => metrics.record_send_ok(signal),
                    Err(SendError::Full(_)) => {
                        metrics.record_send_error(signal, ChannelSendErrorType::Full);
                    }
                    Err(SendError::Closed(_)) => {
                        metrics.record_send_error(signal, ChannelSendErrorType::Closed);
                    }
                }
            }
        }

        result
    }

    /// Attempts to send a message without awaiting.
    pub fn try_send(&self, msg: T) -> Result<(), SendError<T>> {
        let signal = self.signal.and_then(|extract| extract(&msg));
        let result = match &self.inner {
            LocalSenderInner::Mpsc(sender) => sender.send(msg),
            LocalSenderInner::Mpmc(sender) => sender.send(msg),
        };

        if result.is_ok()
            && let Some(queue_depth) = &self.queue_depth
        {
            queue_depth.record_send();
        }
        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.try_borrow_mut() {
                match &result {
                    Ok(()) => metrics.record_send_ok(signal),
                    Err(SendError::Full(_)) => {
                        metrics.record_send_error(signal, ChannelSendErrorType::Full);
                    }
                    Err(SendError::Closed(_)) => {
                        metrics.record_send_error(signal, ChannelSendErrorType::Closed);
                    }
                }
            }
        }

        result
    }
}

enum LocalReceiverInner<T> {
    Mpsc(mpsc::Receiver<T>),
    Mpmc(mpmc::Receiver<T>),
}

/// A generic local channel Receiver.
pub struct LocalReceiver<T> {
    inner: LocalReceiverInner<T>,
    metrics: Option<LocalChannelReceiverMetricsHandle>,
    queue_depth: Option<LocalChannelQueueDepth>,
    signal: Option<fn(&T) -> Option<SignalType>>,
}

impl<T> LocalReceiver<T> {
    /// Creates a new local MPSC receiver.
    #[must_use]
    pub const fn mpsc(receiver: mpsc::Receiver<T>) -> Self {
        Self {
            inner: LocalReceiverInner::Mpsc(receiver),
            metrics: None,
            queue_depth: None,
            signal: None,
        }
    }

    /// Creates a new local MPSC receiver with metrics attached.
    pub(crate) fn mpsc_with_metrics(
        receiver: mpsc::Receiver<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: ChannelReceiverMetricSets,
        capacity: u64,
        queue_depth: LocalChannelQueueDepth,
        signal: Option<fn(&T) -> Option<SignalType>>,
    ) -> Self {
        let handle = Rc::new(RefCell::new(ChannelReceiverMetricsState::new(
            metrics,
            capacity,
            queue_depth.clone(),
        )));
        channel_metrics.register(ChannelMetricsHandle::LocalReceiver(handle.clone()));
        let mut receiver = Self::mpsc(receiver);
        receiver.metrics = Some(handle);
        receiver.queue_depth = Some(queue_depth);
        receiver.signal = signal;
        receiver
    }

    /// Creates a new local MPMC receiver.
    #[must_use]
    pub const fn mpmc(receiver: mpmc::Receiver<T>) -> Self {
        Self {
            inner: LocalReceiverInner::Mpmc(receiver),
            metrics: None,
            queue_depth: None,
            signal: None,
        }
    }

    /// Creates a new local MPMC receiver with metrics attached.
    pub(crate) fn mpmc_with_metrics(
        receiver: mpmc::Receiver<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: ChannelReceiverMetricSets,
        capacity: u64,
        queue_depth: LocalChannelQueueDepth,
        signal: Option<fn(&T) -> Option<SignalType>>,
    ) -> Self {
        let handle = Rc::new(RefCell::new(ChannelReceiverMetricsState::new(
            metrics,
            capacity,
            queue_depth.clone(),
        )));
        channel_metrics.register(ChannelMetricsHandle::LocalReceiver(handle.clone()));
        let mut receiver = Self::mpmc(receiver);
        receiver.metrics = Some(handle);
        receiver.queue_depth = Some(queue_depth);
        receiver.signal = signal;
        receiver
    }

    pub(crate) fn into_mpsc(self) -> Result<mpsc::Receiver<T>, Self> {
        let LocalReceiver {
            inner,
            metrics,
            queue_depth,
            signal,
        } = self;
        match inner {
            LocalReceiverInner::Mpsc(receiver) => Ok(receiver),
            LocalReceiverInner::Mpmc(receiver) => Err(Self {
                inner: LocalReceiverInner::Mpmc(receiver),
                metrics,
                queue_depth,
                signal,
            }),
        }
    }

    /// Receives a message from the channel.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        let result = match &mut self.inner {
            LocalReceiverInner::Mpsc(receiver) => receiver.recv().await,
            LocalReceiverInner::Mpmc(receiver) => receiver.recv().await,
        };

        if result.is_ok() {
            if let Some(queue_depth) = &self.queue_depth {
                queue_depth.record_receive();
            }
        }
        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.try_borrow_mut() {
                if let Ok(message) = &result {
                    metrics.record_recv_ok(self.signal.and_then(|extract| extract(message)));
                }
            }
        }

        result
    }

    /// Tries to receive a message from the channel.
    pub fn try_recv(&mut self) -> Result<T, RecvError> {
        let result = match &mut self.inner {
            LocalReceiverInner::Mpsc(receiver) => receiver.try_recv(),
            LocalReceiverInner::Mpmc(receiver) => receiver.try_recv(),
        };

        if result.is_ok() {
            if let Some(queue_depth) = &self.queue_depth {
                queue_depth.record_receive();
            }
        }
        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.try_borrow_mut() {
                if let Ok(message) = &result {
                    metrics.record_recv_ok(self.signal.and_then(|extract| extract(message)));
                }
            }
        }

        result
    }

    /// Checks if the channel is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match &self.inner {
            LocalReceiverInner::Mpsc(receiver) => receiver.is_empty(),
            LocalReceiverInner::Mpmc(receiver) => receiver.is_empty(),
        }
    }

    /// Returns `true` once all senders are gone or the channel has been
    /// explicitly closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        match &self.inner {
            LocalReceiverInner::Mpsc(receiver) => receiver.is_closed(),
            LocalReceiverInner::Mpmc(receiver) => receiver.is_closed(),
        }
    }
}
