// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Channel-oriented metrics for the OTAP engine.
//!
//! Metrics are split by endpoint role (sender vs receiver). All metrics are scoped
//! using channel endpoint attributes and can be correlated using `channel.id`
//! and `channel.kind`.

use crate::node::NodeId;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[metric_set(name = "channel.sender")]
#[derive(Debug, Default, Clone)]
pub struct ChannelSenderMetrics {
    /// Count of messages successfully sent to the channel.
    #[metric(name = "send.count", unit = "{message}")]
    pub send_count: Counter<u64>,
    /// Count of send failures due to a full channel.
    #[metric(name = "send.error_full", unit = "{1}")]
    pub send_error_full: Counter<u64>,
    /// Count of send failures due to a closed channel.
    #[metric(name = "send.error_closed", unit = "{1}")]
    pub send_error_closed: Counter<u64>,
    // Total bytes successfully sent (when message size is known).
    // TODO: Populate in a future PR when message sizes are tracked.
    // #[metric(name = "send.bytes", unit = "{By}")]
    // pub send_bytes: Counter<u64>,
}

#[metric_set(name = "channel.receiver")]
#[derive(Debug, Default, Clone)]
pub struct ChannelReceiverMetrics {
    /// Count of messages successfully received from the channel.
    #[metric(name = "recv.count", unit = "{message}")]
    pub recv_count: Counter<u64>,
    /// Count of receive attempts when the channel was empty.
    #[metric(name = "recv.error_empty", unit = "{1}")]
    pub recv_error_empty: Counter<u64>,
    /// Count of receive attempts after the channel was closed.
    #[metric(name = "recv.error_closed", unit = "{1}")]
    pub recv_error_closed: Counter<u64>,
    // Total bytes successfully received (when message size is known).
    // TODO: Populate in a future PR when message sizes are tracked.
    // #[metric(name = "recv.bytes", unit = "{By}")]
    // pub recv_bytes: Counter<u64>,
    // Current number of buffered messages.
    // TODO: Populate in a future PR when queue depth is tracked.
    // #[metric(name = "queue.depth", unit = "{message}")]
    // pub queue_depth: Gauge<u64>,
    /// Maximum channel capacity (buffer size).
    #[metric(name = "capacity", unit = "{message}")]
    pub capacity: Gauge<u64>,
}

pub(crate) const CHANNEL_KIND_CONTROL: &str = "control";
pub(crate) const CHANNEL_KIND_PDATA: &str = "pdata";
pub(crate) const CHANNEL_MODE_LOCAL: &str = "local";
pub(crate) const CHANNEL_MODE_SHARED: &str = "shared";
pub(crate) const CHANNEL_TYPE_MPSC: &str = "mpsc";
pub(crate) const CHANNEL_TYPE_MPMC: &str = "mpmc";
pub(crate) const CHANNEL_IMPL_INTERNAL: &str = "internal";
pub(crate) const CHANNEL_IMPL_TOKIO: &str = "tokio";
pub(crate) const CHANNEL_IMPL_FLUME: &str = "flume";

pub(crate) fn control_channel_id(node_id: &NodeId) -> Cow<'static, str> {
    format!("{}:{}", node_id.name, CHANNEL_KIND_CONTROL).into()
}

pub(crate) struct ChannelSenderMetricsState {
    metrics: MetricSet<ChannelSenderMetrics>,
}

impl ChannelSenderMetricsState {
    pub(crate) fn new(metrics: MetricSet<ChannelSenderMetrics>) -> Self {
        Self { metrics }
    }

    #[inline]
    pub(crate) fn record_send_ok(&mut self) {
        self.metrics.send_count.inc();
    }

    #[inline]
    pub(crate) fn record_send_error_full(&mut self) {
        self.metrics.send_error_full.inc();
    }

    #[inline]
    pub(crate) fn record_send_error_closed(&mut self) {
        self.metrics.send_error_closed.inc();
    }

    #[inline]
    pub(crate) fn report(
        &mut self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        metrics_reporter.report(&mut self.metrics)
    }
}

pub(crate) struct ChannelReceiverMetricsState {
    metrics: MetricSet<ChannelReceiverMetrics>,
    capacity: u64,
}

impl ChannelReceiverMetricsState {
    pub(crate) fn new(metrics: MetricSet<ChannelReceiverMetrics>, capacity: u64) -> Self {
        Self { metrics, capacity }
    }

    #[inline]
    pub(crate) fn record_recv_ok(&mut self) {
        self.metrics.recv_count.inc();
    }

    #[inline]
    pub(crate) fn record_recv_error_empty(&mut self) {
        self.metrics.recv_error_empty.inc();
    }

    #[inline]
    pub(crate) fn record_recv_error_closed(&mut self) {
        self.metrics.recv_error_closed.inc();
    }

    #[inline]
    pub(crate) fn report(
        &mut self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        self.metrics.capacity.set(self.capacity);
        metrics_reporter.report(&mut self.metrics)
    }
}

pub(crate) type LocalChannelSenderMetricsHandle = Rc<RefCell<ChannelSenderMetricsState>>;
pub(crate) type LocalChannelReceiverMetricsHandle = Rc<RefCell<ChannelReceiverMetricsState>>;
pub(crate) type SharedChannelSenderMetricsHandle = Arc<Mutex<ChannelSenderMetricsState>>;
pub(crate) type SharedChannelReceiverMetricsHandle = Arc<Mutex<ChannelReceiverMetricsState>>;

#[derive(Clone)]
pub(crate) enum ChannelMetricsHandle {
    LocalSender(LocalChannelSenderMetricsHandle),
    SharedSender(SharedChannelSenderMetricsHandle),
    LocalReceiver(LocalChannelReceiverMetricsHandle),
    SharedReceiver(SharedChannelReceiverMetricsHandle),
}

impl ChannelMetricsHandle {
    #[inline]
    pub(crate) fn report(
        &self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        match self {
            ChannelMetricsHandle::LocalSender(metrics) => match metrics.try_borrow_mut() {
                Ok(mut metrics) => metrics.report(metrics_reporter),
                Err(_) => Ok(()),
            },
            ChannelMetricsHandle::SharedSender(metrics) => match metrics.lock() {
                Ok(mut metrics) => metrics.report(metrics_reporter),
                Err(_) => Ok(()),
            },
            ChannelMetricsHandle::LocalReceiver(metrics) => match metrics.try_borrow_mut() {
                Ok(mut metrics) => metrics.report(metrics_reporter),
                Err(_) => Ok(()),
            },
            ChannelMetricsHandle::SharedReceiver(metrics) => match metrics.lock() {
                Ok(mut metrics) => metrics.report(metrics_reporter),
                Err(_) => Ok(()),
            },
        }
    }
}

#[derive(Default)]
pub(crate) struct ChannelMetricsRegistry {
    handles: Vec<ChannelMetricsHandle>,
}

impl ChannelMetricsRegistry {
    pub(crate) fn register(&mut self, handle: ChannelMetricsHandle) {
        self.handles.push(handle);
    }

    pub(crate) fn into_handles(self) -> Vec<ChannelMetricsHandle> {
        self.handles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ControllerContext, PipelineContext};
    use crate::local::message::{LocalReceiver, LocalSender};
    use otap_df_channel::error::{RecvError, SendError};
    use otap_df_channel::mpsc;
    use otap_df_config::node::NodeKind;
    use otap_df_telemetry::metrics::MetricSetHandler;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;

    fn test_context() -> PipelineContext {
        let registry = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry);
        controller_ctx
            .pipeline_context_with("grp".into(), "pipe".into(), 0, 0)
            .with_node_context("node".into(), "urn:test".into(), NodeKind::Receiver)
    }

    fn take_local_sender_handle(
        handles: &[ChannelMetricsHandle],
    ) -> LocalChannelSenderMetricsHandle {
        handles
            .iter()
            .find_map(|handle| match handle {
                ChannelMetricsHandle::LocalSender(handle) => Some(handle.clone()),
                _ => None,
            })
            .expect("missing local sender metrics handle")
    }

    fn take_local_receiver_handle(
        handles: &[ChannelMetricsHandle],
    ) -> LocalChannelReceiverMetricsHandle {
        handles
            .iter()
            .find_map(|handle| match handle {
                ChannelMetricsHandle::LocalReceiver(handle) => Some(handle.clone()),
                _ => None,
            })
            .expect("missing local receiver metrics handle")
    }

    #[test]
    fn channel_sender_metrics_record_send_outcomes() {
        let pipeline_ctx = test_context();
        let mut registry = ChannelMetricsRegistry::default();
        let (sender, receiver) = mpsc::Channel::new(1);
        let sender = LocalSender::mpsc_with_metrics(
            sender,
            &pipeline_ctx,
            &mut registry,
            "test:sender".into(),
            CHANNEL_KIND_PDATA,
        );
        sender.try_send(1).unwrap();
        assert!(matches!(sender.try_send(2), Err(SendError::Full(_))));
        drop(receiver);
        assert!(matches!(sender.try_send(3), Err(SendError::Closed(_))));

        let handles = registry.into_handles();
        let sender_handle = take_local_sender_handle(&handles);
        let metrics = sender_handle.borrow();
        assert_eq!(metrics.metrics.send_count.get(), 1);
        assert_eq!(metrics.metrics.send_error_full.get(), 1);
        assert_eq!(metrics.metrics.send_error_closed.get(), 1);
    }

    #[test]
    fn channel_receiver_metrics_record_recv_outcomes_and_capacity() {
        let pipeline_ctx = test_context();
        let mut registry = ChannelMetricsRegistry::default();
        let (sender, receiver) = mpsc::Channel::new(1);
        let sender = LocalSender::mpsc(sender);
        let mut receiver = LocalReceiver::mpsc_with_metrics(
            receiver,
            &pipeline_ctx,
            &mut registry,
            "test:receiver".into(),
            CHANNEL_KIND_PDATA,
            1,
        );

        assert!(matches!(receiver.try_recv(), Err(RecvError::Empty)));
        sender.try_send(1).unwrap();
        assert!(matches!(receiver.try_recv(), Ok(1)));
        drop(sender);
        assert!(matches!(receiver.try_recv(), Err(RecvError::Closed)));

        let handles = registry.into_handles();
        let receiver_handle = take_local_receiver_handle(&handles);
        {
            let metrics = receiver_handle.borrow();
            assert_eq!(metrics.metrics.recv_count.get(), 1);
            assert_eq!(metrics.metrics.recv_error_empty.get(), 1);
            assert_eq!(metrics.metrics.recv_error_closed.get(), 1);
        }
        let (snapshot_rx, mut reporter) = MetricsReporter::create_new_and_receiver(1);
        receiver_handle.borrow_mut().report(&mut reporter).unwrap();
        let snapshot = snapshot_rx.recv().unwrap();
        let descriptor = ChannelReceiverMetrics::default().descriptor();
        let capacity_index = descriptor
            .metrics
            .iter()
            .position(|field| field.name == "capacity")
            .expect("capacity metric not found");
        assert_eq!(snapshot.get_metrics()[capacity_index].to_u64_lossy(), 1);
    }
}
