// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Abstraction to represent generic local senders and receivers.

use crate::channel_metrics::{
    CHANNEL_IMPL_INTERNAL, CHANNEL_MODE_LOCAL, CHANNEL_TYPE_MPMC, CHANNEL_TYPE_MPSC,
    ChannelMetricsHandle, ChannelMetricsRegistry, ChannelReceiverMetrics,
    ChannelReceiverMetricsState, ChannelSenderMetrics, ChannelSenderMetricsState,
    LocalChannelReceiverMetricsHandle, LocalChannelSenderMetricsHandle,
};
use crate::context::PipelineContext;
use otap_df_channel::error::{RecvError, SendError};
use otap_df_channel::{mpmc, mpsc};
use std::borrow::Cow;
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
        }
    }
}

impl<T> LocalSender<T> {
    /// Creates a new local MPSC sender.
    pub fn mpsc(sender: mpsc::Sender<T>) -> Self {
        Self {
            inner: LocalSenderInner::Mpsc(sender),
            metrics: None,
        }
    }

    /// Creates a new local MPSC sender with metrics attached.
    pub(crate) fn mpsc_with_metrics(
        sender: mpsc::Sender<T>,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
    ) -> Self {
        let attrs = pipeline_ctx.channel_attribute_set(
            channel_id,
            channel_kind,
            CHANNEL_MODE_LOCAL,
            CHANNEL_TYPE_MPSC,
            CHANNEL_IMPL_INTERNAL,
        );
        let metrics = pipeline_ctx.register_metric_set_with_attrs::<ChannelSenderMetrics>(attrs);
        let handle = Rc::new(RefCell::new(ChannelSenderMetricsState::new(metrics)));
        channel_metrics.register(ChannelMetricsHandle::LocalSender(handle.clone()));
        let mut sender = Self::mpsc(sender);
        sender.metrics = Some(handle);
        sender
    }

    /// Creates a new local MPMC sender.
    pub fn mpmc(sender: mpmc::Sender<T>) -> Self {
        Self {
            inner: LocalSenderInner::Mpmc(sender),
            metrics: None,
        }
    }

    /// Creates a new local MPMC sender with metrics attached.
    pub(crate) fn mpmc_with_metrics(
        sender: mpmc::Sender<T>,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
    ) -> Self {
        let attrs = pipeline_ctx.channel_attribute_set(
            channel_id,
            channel_kind,
            CHANNEL_MODE_LOCAL,
            CHANNEL_TYPE_MPMC,
            CHANNEL_IMPL_INTERNAL,
        );
        let metrics = pipeline_ctx.register_metric_set_with_attrs::<ChannelSenderMetrics>(attrs);
        let handle = Rc::new(RefCell::new(ChannelSenderMetricsState::new(metrics)));
        channel_metrics.register(ChannelMetricsHandle::LocalSender(handle.clone()));
        let mut sender = Self::mpmc(sender);
        sender.metrics = Some(handle);
        sender
    }

    pub(crate) fn into_mpsc(self) -> Result<mpsc::Sender<T>, Self> {
        let LocalSender { inner, metrics } = self;
        match inner {
            LocalSenderInner::Mpsc(sender) => Ok(sender),
            LocalSenderInner::Mpmc(sender) => Err(Self {
                inner: LocalSenderInner::Mpmc(sender),
                metrics,
            }),
        }
    }

    /// Sends a message to the channel.
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        let result = match &self.inner {
            LocalSenderInner::Mpsc(sender) => sender.send_async(msg).await,
            LocalSenderInner::Mpmc(sender) => sender.send_async(msg).await,
        };

        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.try_borrow_mut() {
                match &result {
                    Ok(()) => metrics.record_send_ok(),
                    Err(SendError::Full(_)) => metrics.record_send_error_full(),
                    Err(SendError::Closed(_)) => metrics.record_send_error_closed(),
                }
            }
        }

        result
    }

    /// Attempts to send a message without awaiting.
    pub fn try_send(&self, msg: T) -> Result<(), SendError<T>> {
        let result = match &self.inner {
            LocalSenderInner::Mpsc(sender) => sender.send(msg),
            LocalSenderInner::Mpmc(sender) => sender.send(msg),
        };

        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.try_borrow_mut() {
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

enum LocalReceiverInner<T> {
    Mpsc(mpsc::Receiver<T>),
    Mpmc(mpmc::Receiver<T>),
}

/// A generic local channel Receiver.
pub struct LocalReceiver<T> {
    inner: LocalReceiverInner<T>,
    metrics: Option<LocalChannelReceiverMetricsHandle>,
}

impl<T> LocalReceiver<T> {
    /// Creates a new local MPSC receiver.
    #[must_use]
    pub fn mpsc(receiver: mpsc::Receiver<T>) -> Self {
        Self {
            inner: LocalReceiverInner::Mpsc(receiver),
            metrics: None,
        }
    }

    /// Creates a new local MPSC receiver with metrics attached.
    pub(crate) fn mpsc_with_metrics(
        receiver: mpsc::Receiver<T>,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
        capacity: u64,
    ) -> Self {
        let attrs = pipeline_ctx.channel_attribute_set(
            channel_id,
            channel_kind,
            CHANNEL_MODE_LOCAL,
            CHANNEL_TYPE_MPSC,
            CHANNEL_IMPL_INTERNAL,
        );
        let metrics = pipeline_ctx.register_metric_set_with_attrs::<ChannelReceiverMetrics>(attrs);
        let handle = Rc::new(RefCell::new(ChannelReceiverMetricsState::new(
            metrics, capacity,
        )));
        channel_metrics.register(ChannelMetricsHandle::LocalReceiver(handle.clone()));
        let mut receiver = Self::mpsc(receiver);
        receiver.metrics = Some(handle);
        receiver
    }

    /// Creates a new local MPMC receiver.
    #[must_use]
    pub fn mpmc(receiver: mpmc::Receiver<T>) -> Self {
        Self {
            inner: LocalReceiverInner::Mpmc(receiver),
            metrics: None,
        }
    }

    /// Creates a new local MPMC receiver with metrics attached.
    pub(crate) fn mpmc_with_metrics(
        receiver: mpmc::Receiver<T>,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
        capacity: u64,
    ) -> Self {
        let attrs = pipeline_ctx.channel_attribute_set(
            channel_id,
            channel_kind,
            CHANNEL_MODE_LOCAL,
            CHANNEL_TYPE_MPMC,
            CHANNEL_IMPL_INTERNAL,
        );
        let metrics = pipeline_ctx.register_metric_set_with_attrs::<ChannelReceiverMetrics>(attrs);
        let handle = Rc::new(RefCell::new(ChannelReceiverMetricsState::new(
            metrics, capacity,
        )));
        channel_metrics.register(ChannelMetricsHandle::LocalReceiver(handle.clone()));
        let mut receiver = Self::mpmc(receiver);
        receiver.metrics = Some(handle);
        receiver
    }

    pub(crate) fn into_mpsc(self) -> Result<mpsc::Receiver<T>, Self> {
        let LocalReceiver { inner, metrics } = self;
        match inner {
            LocalReceiverInner::Mpsc(receiver) => Ok(receiver),
            LocalReceiverInner::Mpmc(receiver) => Err(Self {
                inner: LocalReceiverInner::Mpmc(receiver),
                metrics,
            }),
        }
    }

    /// Receives a message from the channel.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        let result = match &mut self.inner {
            LocalReceiverInner::Mpsc(receiver) => receiver.recv().await,
            LocalReceiverInner::Mpmc(receiver) => receiver.recv().await,
        };

        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.try_borrow_mut() {
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
            LocalReceiverInner::Mpsc(receiver) => receiver.try_recv(),
            LocalReceiverInner::Mpmc(receiver) => receiver.try_recv(),
        };

        if let Some(metrics) = &self.metrics {
            if let Ok(mut metrics) = metrics.try_borrow_mut() {
                match &result {
                    Ok(_) => metrics.record_recv_ok(),
                    Err(RecvError::Empty) => metrics.record_recv_error_empty(),
                    Err(RecvError::Closed) => metrics.record_recv_error_closed(),
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
}
