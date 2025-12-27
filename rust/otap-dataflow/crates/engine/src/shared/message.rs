// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Abstraction to represent generic shared senders and receivers.

use crate::channel_metrics::{
    CHANNEL_IMPL_FLUME, CHANNEL_IMPL_TOKIO, CHANNEL_MODE_SHARED, CHANNEL_TYPE_MPMC,
    CHANNEL_TYPE_MPSC, ChannelMetricsHandle, ChannelMetricsRegistry, ChannelReceiverMetrics,
    ChannelReceiverMetricsState, ChannelSenderMetrics, ChannelSenderMetricsState,
    SharedChannelReceiverMetricsHandle, SharedChannelSenderMetricsHandle,
};
use crate::context::PipelineContext;
use otap_df_channel::error::{RecvError, SendError};
use std::borrow::Cow;
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
        }
    }
}

impl<T> SharedSender<T> {
    /// Creates a new shared MPSC sender.
    pub fn mpsc(sender: tokio::sync::mpsc::Sender<T>) -> Self {
        Self {
            inner: SharedSenderInner::Mpsc(sender),
            metrics: None,
        }
    }

    /// Creates a new shared MPSC sender with metrics attached.
    pub(crate) fn mpsc_with_metrics(
        sender: tokio::sync::mpsc::Sender<T>,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
    ) -> Self {
        let attrs = pipeline_ctx.channel_attribute_set(
            channel_id,
            channel_kind,
            CHANNEL_MODE_SHARED,
            CHANNEL_TYPE_MPSC,
            CHANNEL_IMPL_TOKIO,
        );
        let metrics = pipeline_ctx
            .metrics_registry()
            .register::<ChannelSenderMetrics>(attrs);
        let handle = Arc::new(Mutex::new(ChannelSenderMetricsState::new(metrics)));
        channel_metrics.register(ChannelMetricsHandle::SharedSender(handle.clone()));
        let mut sender = Self::mpsc(sender);
        sender.set_metrics(handle);
        sender
    }

    /// Creates a new shared MPMC sender.
    pub fn mpmc(sender: flume::Sender<T>) -> Self {
        Self {
            inner: SharedSenderInner::Mpmc(sender),
            metrics: None,
        }
    }

    /// Creates a new shared MPMC sender with metrics attached.
    pub(crate) fn mpmc_with_metrics(
        sender: flume::Sender<T>,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
    ) -> Self {
        let attrs = pipeline_ctx.channel_attribute_set(
            channel_id,
            channel_kind,
            CHANNEL_MODE_SHARED,
            CHANNEL_TYPE_MPMC,
            CHANNEL_IMPL_FLUME,
        );
        let metrics = pipeline_ctx
            .metrics_registry()
            .register::<ChannelSenderMetrics>(attrs);
        let handle = Arc::new(Mutex::new(ChannelSenderMetricsState::new(metrics)));
        channel_metrics.register(ChannelMetricsHandle::SharedSender(handle.clone()));
        let mut sender = Self::mpmc(sender);
        sender.set_metrics(handle);
        sender
    }

    /// Attaches metrics to this sender.
    pub(crate) fn set_metrics(&mut self, metrics: SharedChannelSenderMetricsHandle) {
        self.metrics = Some(metrics);
    }

    pub(crate) fn into_mpsc(self) -> Result<tokio::sync::mpsc::Sender<T>, Self> {
        let SharedSender { inner, metrics } = self;
        match inner {
            SharedSenderInner::Mpsc(sender) => Ok(sender),
            SharedSenderInner::Mpmc(sender) => Err(Self {
                inner: SharedSenderInner::Mpmc(sender),
                metrics,
            }),
        }
    }

    /// Sends a message to the channel.
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        let result = match &self.inner {
            SharedSenderInner::Mpsc(sender) => {
                sender.send(msg).await.map_err(|e| SendError::Closed(e.0))
            }
            SharedSenderInner::Mpmc(sender) => sender.send(msg).map_err(|e| SendError::Closed(e.0)),
        };

        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
                match &result {
                    Ok(()) => metrics.record_send_ok(),
                    Err(SendError::Full(_)) => metrics.record_send_error_full(),
                    Err(SendError::Closed(_)) => metrics.record_send_error_closed(),
                }
            }
        }

        result
    }

    /// Attempts to send a message to the channel without awaiting.
    pub fn try_send(&self, msg: T) -> Result<(), SendError<T>> {
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

        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
                match &result {
                    Ok(()) => metrics.record_send_ok(),
                    Err(SendError::Full(_)) => metrics.record_send_error_full(),
                    Err(SendError::Closed(_)) => metrics.record_send_error_closed(),
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
}

impl<T> SharedReceiver<T> {
    /// Creates a new shared MPSC receiver.
    #[must_use]
    pub fn mpsc(receiver: tokio::sync::mpsc::Receiver<T>) -> Self {
        Self {
            inner: SharedReceiverInner::Mpsc(receiver),
            metrics: None,
        }
    }

    /// Creates a new shared MPSC receiver with metrics attached.
    pub(crate) fn mpsc_with_metrics(
        receiver: tokio::sync::mpsc::Receiver<T>,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
        capacity: u64,
    ) -> Self {
        let attrs = pipeline_ctx.channel_attribute_set(
            channel_id,
            channel_kind,
            CHANNEL_MODE_SHARED,
            CHANNEL_TYPE_MPSC,
            CHANNEL_IMPL_TOKIO,
        );
        let metrics = pipeline_ctx
            .metrics_registry()
            .register::<ChannelReceiverMetrics>(attrs);
        let handle = Arc::new(Mutex::new(ChannelReceiverMetricsState::new(
            metrics, capacity,
        )));
        channel_metrics.register(ChannelMetricsHandle::SharedReceiver(handle.clone()));
        let mut receiver = Self::mpsc(receiver);
        receiver.set_metrics(handle);
        receiver
    }

    /// Creates a new shared MPMC receiver.
    #[must_use]
    pub fn mpmc(receiver: flume::Receiver<T>) -> Self {
        Self {
            inner: SharedReceiverInner::Mpmc(receiver),
            metrics: None,
        }
    }

    /// Creates a new shared MPMC receiver with metrics attached.
    pub(crate) fn mpmc_with_metrics(
        receiver: flume::Receiver<T>,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
        capacity: u64,
    ) -> Self {
        let attrs = pipeline_ctx.channel_attribute_set(
            channel_id,
            channel_kind,
            CHANNEL_MODE_SHARED,
            CHANNEL_TYPE_MPMC,
            CHANNEL_IMPL_FLUME,
        );
        let metrics = pipeline_ctx
            .metrics_registry()
            .register::<ChannelReceiverMetrics>(attrs);
        let handle = Arc::new(Mutex::new(ChannelReceiverMetricsState::new(
            metrics, capacity,
        )));
        channel_metrics.register(ChannelMetricsHandle::SharedReceiver(handle.clone()));
        let mut receiver = Self::mpmc(receiver);
        receiver.set_metrics(handle);
        receiver
    }

    /// Attaches metrics to this receiver.
    pub(crate) fn set_metrics(&mut self, metrics: SharedChannelReceiverMetricsHandle) {
        self.metrics = Some(metrics);
    }

    pub(crate) fn into_mpsc(self) -> Result<tokio::sync::mpsc::Receiver<T>, Self> {
        let SharedReceiver { inner, metrics } = self;
        match inner {
            SharedReceiverInner::Mpsc(receiver) => Ok(receiver),
            SharedReceiverInner::Mpmc(receiver) => Err(Self {
                inner: SharedReceiverInner::Mpmc(receiver),
                metrics,
            }),
        }
    }

    /// Receives a message from the channel.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        let result = match &mut self.inner {
            SharedReceiverInner::Mpsc(receiver) => receiver.recv().await.ok_or(RecvError::Closed),
            SharedReceiverInner::Mpmc(receiver) => receiver.recv().map_err(|_| RecvError::Closed),
        };

        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
                match &result {
                    Ok(_) => metrics.record_recv_ok(),
                    Err(RecvError::Empty) => metrics.record_recv_error_empty(),
                    Err(RecvError::Closed) => metrics.record_recv_error_closed(),
                }
            }
        }

        result
    }

    /// Tries to receive a message from the channel.
    pub fn try_recv(&mut self) -> Result<T, RecvError> {
        let result = match &mut self.inner {
            SharedReceiverInner::Mpsc(receiver) => {
                receiver.try_recv().map_err(|_| RecvError::Closed)
            }
            SharedReceiverInner::Mpmc(receiver) => {
                receiver.try_recv().map_err(|_| RecvError::Closed)
            }
        };

        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
                match &result {
                    Ok(_) => metrics.record_recv_ok(),
                    Err(RecvError::Empty) => metrics.record_recv_error_empty(),
                    Err(RecvError::Closed) => metrics.record_recv_error_closed(),
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
}
