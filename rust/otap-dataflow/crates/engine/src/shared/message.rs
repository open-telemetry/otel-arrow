// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Abstraction to represent generic shared senders and receivers.

use crate::channel_metrics::{
    ChannelMetricsHandle, ChannelMetricsRegistry, ChannelQueueDepth, ChannelReceiverMetricSets,
    ChannelReceiverMetricsState, ChannelSendErrorType, ChannelSenderMetricSets,
    ChannelSenderMetricsState, SharedChannelQueueDepth, SharedChannelReceiverMetricsHandle,
    SharedChannelSenderMetricsHandle,
};
use otap_df_channel::error::{RecvError, SendError};
use otap_df_config::SignalType;
use std::sync::{Arc, Mutex};

enum SharedSenderInner<T> {
    Mpsc(tokio::sync::mpsc::Sender<T>),
    Mpmc(flume::Sender<T>),
}

/// A generic shared channel Sender.
#[must_use = "A `Sender` is requested but not used."]
pub struct SharedSender<T> {
    inner: SharedSenderInner<T>,
    metrics: Option<SharedChannelSenderMetricsHandle>,
    queue_depth: Option<SharedChannelQueueDepth>,
    signal: Option<fn(&T) -> Option<SignalType>>,
}

impl<T> Clone for SharedSender<T> {
    fn clone(&self) -> Self {
        let inner = match &self.inner {
            SharedSenderInner::Mpsc(sender) => SharedSenderInner::Mpsc(sender.clone()),
            SharedSenderInner::Mpmc(sender) => SharedSenderInner::Mpmc(sender.clone()),
        };
        Self {
            inner,
            metrics: self.metrics.clone(),
            queue_depth: self.queue_depth.clone(),
            signal: self.signal,
        }
    }
}

impl<T> SharedSender<T> {
    /// Creates a new shared MPSC sender.
    pub const fn mpsc(sender: tokio::sync::mpsc::Sender<T>) -> Self {
        Self {
            inner: SharedSenderInner::Mpsc(sender),
            metrics: None,
            queue_depth: None,
            signal: None,
        }
    }

    /// Creates a new shared MPSC sender with metrics attached.
    pub(crate) fn mpsc_with_metrics(
        sender: tokio::sync::mpsc::Sender<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: ChannelSenderMetricSets,
        queue_depth: SharedChannelQueueDepth,
        signal: Option<fn(&T) -> Option<SignalType>>,
    ) -> Self {
        let handle = Arc::new(Mutex::new(ChannelSenderMetricsState::new(metrics)));
        channel_metrics.register(ChannelMetricsHandle::SharedSender(handle.clone()));
        let mut sender = Self::mpsc(sender);
        sender.metrics = Some(handle);
        sender.queue_depth = Some(queue_depth);
        sender.signal = signal;
        sender
    }

    /// Creates a new shared MPMC sender.
    pub const fn mpmc(sender: flume::Sender<T>) -> Self {
        Self {
            inner: SharedSenderInner::Mpmc(sender),
            metrics: None,
            queue_depth: None,
            signal: None,
        }
    }

    /// Creates a new shared MPMC sender with metrics attached.
    pub(crate) fn mpmc_with_metrics(
        sender: flume::Sender<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: ChannelSenderMetricSets,
        queue_depth: SharedChannelQueueDepth,
        signal: Option<fn(&T) -> Option<SignalType>>,
    ) -> Self {
        let handle = Arc::new(Mutex::new(ChannelSenderMetricsState::new(metrics)));
        channel_metrics.register(ChannelMetricsHandle::SharedSender(handle.clone()));
        let mut sender = Self::mpmc(sender);
        sender.metrics = Some(handle);
        sender.queue_depth = Some(queue_depth);
        sender.signal = signal;
        sender
    }

    pub(crate) fn into_mpsc(self) -> Result<tokio::sync::mpsc::Sender<T>, Self> {
        let SharedSender {
            inner,
            metrics,
            queue_depth,
            signal,
        } = self;
        match inner {
            SharedSenderInner::Mpsc(sender) => Ok(sender),
            SharedSenderInner::Mpmc(sender) => Err(Self {
                inner: SharedSenderInner::Mpmc(sender),
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
            SharedSenderInner::Mpsc(sender) => {
                sender.send(msg).await.map_err(|e| SendError::Closed(e.0))
            }
            SharedSenderInner::Mpmc(sender) => sender
                .send_async(msg)
                .await
                .map_err(|e| SendError::Closed(e.0)),
        };

        if result.is_ok()
            && let Some(queue_depth) = &self.queue_depth
        {
            queue_depth.record_send();
        }
        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
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

    /// Attempts to send a message to the channel without awaiting.
    pub fn try_send(&self, msg: T) -> Result<(), SendError<T>> {
        let signal = self.signal.and_then(|extract| extract(&msg));
        let result = match &self.inner {
            SharedSenderInner::Mpsc(sender) => sender.try_send(msg).map_err(|e| match e {
                tokio::sync::mpsc::error::TrySendError::Full(v) => SendError::Full(v),
                tokio::sync::mpsc::error::TrySendError::Closed(v) => SendError::Closed(v),
            }),
            SharedSenderInner::Mpmc(sender) => sender.try_send(msg).map_err(|e| match e {
                flume::TrySendError::Full(v) => SendError::Full(v),
                flume::TrySendError::Disconnected(v) => SendError::Closed(v),
            }),
        };

        if result.is_ok()
            && let Some(queue_depth) = &self.queue_depth
        {
            queue_depth.record_send();
        }
        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
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

enum SharedReceiverInner<T> {
    Mpsc(tokio::sync::mpsc::Receiver<T>),
    Mpmc(flume::Receiver<T>),
}

/// A generic shared channel Receiver.
pub struct SharedReceiver<T> {
    inner: SharedReceiverInner<T>,
    metrics: Option<SharedChannelReceiverMetricsHandle>,
    queue_depth: Option<SharedChannelQueueDepth>,
    signal: Option<fn(&T) -> Option<SignalType>>,
}

impl<T> SharedReceiver<T> {
    /// Creates a new shared MPSC receiver.
    #[must_use]
    pub const fn mpsc(receiver: tokio::sync::mpsc::Receiver<T>) -> Self {
        Self {
            inner: SharedReceiverInner::Mpsc(receiver),
            metrics: None,
            queue_depth: None,
            signal: None,
        }
    }

    /// Creates a new shared MPSC receiver with metrics attached.
    pub(crate) fn mpsc_with_metrics(
        receiver: tokio::sync::mpsc::Receiver<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: ChannelReceiverMetricSets,
        capacity: u64,
        queue_depth: SharedChannelQueueDepth,
        signal: Option<fn(&T) -> Option<SignalType>>,
    ) -> Self {
        let handle = Arc::new(Mutex::new(ChannelReceiverMetricsState::new(
            metrics,
            capacity,
            queue_depth.clone(),
        )));
        channel_metrics.register(ChannelMetricsHandle::SharedReceiver(handle.clone()));
        let mut receiver = Self::mpsc(receiver);
        receiver.metrics = Some(handle);
        receiver.queue_depth = Some(queue_depth);
        receiver.signal = signal;
        receiver
    }

    /// Creates a new shared MPMC receiver.
    #[must_use]
    pub const fn mpmc(receiver: flume::Receiver<T>) -> Self {
        Self {
            inner: SharedReceiverInner::Mpmc(receiver),
            metrics: None,
            queue_depth: None,
            signal: None,
        }
    }

    /// Creates a new shared MPMC receiver with metrics attached.
    pub(crate) fn mpmc_with_metrics(
        receiver: flume::Receiver<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: ChannelReceiverMetricSets,
        capacity: u64,
        queue_depth: SharedChannelQueueDepth,
        signal: Option<fn(&T) -> Option<SignalType>>,
    ) -> Self {
        let handle = Arc::new(Mutex::new(ChannelReceiverMetricsState::new(
            metrics,
            capacity,
            queue_depth.clone(),
        )));
        channel_metrics.register(ChannelMetricsHandle::SharedReceiver(handle.clone()));
        let mut receiver = Self::mpmc(receiver);
        receiver.metrics = Some(handle);
        receiver.queue_depth = Some(queue_depth);
        receiver.signal = signal;
        receiver
    }

    pub(crate) fn into_mpsc(self) -> Result<tokio::sync::mpsc::Receiver<T>, Self> {
        let SharedReceiver {
            inner,
            metrics,
            queue_depth,
            signal,
        } = self;
        match inner {
            SharedReceiverInner::Mpsc(receiver) => Ok(receiver),
            SharedReceiverInner::Mpmc(receiver) => Err(Self {
                inner: SharedReceiverInner::Mpmc(receiver),
                metrics,
                queue_depth,
                signal,
            }),
        }
    }

    /// Receives a message from the channel.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        let result = match &mut self.inner {
            SharedReceiverInner::Mpsc(receiver) => receiver.recv().await.ok_or(RecvError::Closed),
            SharedReceiverInner::Mpmc(receiver) => {
                receiver.recv_async().await.map_err(|_| RecvError::Closed)
            }
        };

        if result.is_ok()
            && let Some(queue_depth) = &self.queue_depth
        {
            queue_depth.record_receive();
        }
        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
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
            SharedReceiverInner::Mpsc(receiver) => receiver.try_recv().map_err(|e| match e {
                tokio::sync::mpsc::error::TryRecvError::Empty => RecvError::Empty,
                tokio::sync::mpsc::error::TryRecvError::Disconnected => RecvError::Closed,
            }),
            SharedReceiverInner::Mpmc(receiver) => receiver.try_recv().map_err(|e| match e {
                flume::TryRecvError::Empty => RecvError::Empty,
                flume::TryRecvError::Disconnected => RecvError::Closed,
            }),
        };

        if result.is_ok()
            && let Some(queue_depth) = &self.queue_depth
        {
            queue_depth.record_receive();
        }
        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
                if let Ok(message) = &result {
                    metrics.record_recv_ok(self.signal.and_then(|extract| extract(message)));
                }
            }
        }

        result
    }

    /// Returns `true` if the channel is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match &self.inner {
            SharedReceiverInner::Mpsc(receiver) => receiver.is_empty(),
            SharedReceiverInner::Mpmc(receiver) => receiver.is_empty(),
        }
    }

    /// Returns `true` once all senders are gone or the channel has been
    /// explicitly closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        match &self.inner {
            SharedReceiverInner::Mpsc(receiver) => receiver.is_closed(),
            SharedReceiverInner::Mpmc(receiver) => receiver.is_disconnected(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_channel::error::RecvError;

    #[test]
    fn test_mpsc_try_recv_empty_returns_empty_not_closed() {
        let (tx, rx) = tokio::sync::mpsc::channel::<String>(10);
        let mut receiver = SharedReceiver::mpsc(rx);

        // Channel is empty but sender is alive -- should return Empty
        let result = receiver.try_recv();
        assert!(
            matches!(result, Err(RecvError::Empty)),
            "expected Empty, got {result:?}"
        );

        drop(tx);

        // Now channel is closed -- should return Closed
        let result = receiver.try_recv();
        assert!(
            matches!(result, Err(RecvError::Closed)),
            "expected Closed, got {result:?}"
        );
    }

    #[test]
    fn test_mpmc_try_recv_empty_returns_empty_not_closed() {
        let (tx, rx) = flume::bounded::<String>(10);
        let mut receiver = SharedReceiver::mpmc(rx);

        // Channel is empty but sender is alive -- should return Empty
        let result = receiver.try_recv();
        assert!(
            matches!(result, Err(RecvError::Empty)),
            "expected Empty, got {result:?}"
        );

        drop(tx);

        // Now channel is closed -- should return Closed
        let result = receiver.try_recv();
        assert!(
            matches!(result, Err(RecvError::Closed)),
            "expected Closed, got {result:?}"
        );
    }

    /// Regression test for https://github.com/open-telemetry/otel-arrow/issues/1704.
    #[test]
    fn test_mpmc_recv_send_do_not_block_runtime_thread() {
        use std::sync::mpsc;
        use std::time::Duration;

        let (done_tx, done_rx) = mpsc::channel();

        let worker = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("failed to build current-thread runtime");

            let outcome = rt.block_on(async {
                let (tx, rx) = flume::bounded::<String>(1);
                let mut receiver = SharedReceiver::mpmc(rx);
                let sender = SharedSender::mpmc(tx);

                // On a single-threaded runtime `recv` is polled first and must
                // yield (Pending) so `send` can make progress on the same thread.
                // A blocking flume `recv()` would deadlock here.
                tokio::join!(async { receiver.recv().await }, async {
                    sender.send("hello".to_owned()).await
                },)
            });

            let _ = done_tx.send(outcome);
        });

        match done_rx.recv_timeout(Duration::from_secs(5)) {
            Ok((received, sent)) => {
                worker.join().expect("worker thread panicked");
                assert!(sent.is_ok(), "send failed: {sent:?}");
                assert_eq!(received.expect("recv failed"), "hello");
            }
            Err(_) => panic!(
                "shared MPMC recv()/send() blocked the runtime thread; flume's \
                 sync interface must not be used inside async fns (issue #1704)"
            ),
        }
    }
}
