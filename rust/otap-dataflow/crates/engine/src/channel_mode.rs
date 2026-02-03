// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for wiring control channels across local/shared nodes.
//!
//! Rationale:
//! - The receiver/processor/exporter wrappers all perform identical control-channel wiring.
//!   They register the channel entity, attach metrics if enabled, and preserve the original
//!   channel types (local vs shared).
//! - Duplicating that logic makes changes error-prone and forces three places to stay in sync.
//!   `ChannelMode` centralizes the wiring while still compiling down to mode-specific code.
//! - The helper is monomorphized for `LocalMode` and `SharedMode`, so there is no dynamic
//!   dispatch on the hot path. This approach improves maintainability with negligible runtime cost.
//! - Keeping the mode-specific sender/receiver types preserves `Send` requirements for shared
//!   nodes while still avoiding local/shared code divergence.

use crate::channel_metrics::{
    CHANNEL_IMPL_INTERNAL, CHANNEL_IMPL_TOKIO, CHANNEL_KIND_CONTROL, CHANNEL_MODE_LOCAL,
    CHANNEL_MODE_SHARED, CHANNEL_TYPE_MPSC, ChannelMetricsRegistry, ChannelReceiverMetrics,
    ChannelSenderMetrics, control_channel_id,
};
use crate::context::PipelineContext;
use crate::control::NodeControlMsg;
use crate::entity_context::current_node_telemetry_handle;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_channel::mpsc;
use otap_df_telemetry::metrics::MetricSet;

pub(crate) trait ChannelMode {
    const CHANNEL_MODE: &'static str;
    const CHANNEL_IMPL: &'static str;

    type ControlSender<T>;
    type ControlReceiver<T>;
    type InnerSender<T>;
    type InnerReceiver<T>;

    /// Returns the original sender when it is already wrapped.
    /// This allows attaching metrics only once without losing existing wrappers.
    fn try_into_inner_sender<T>(
        sender: Self::ControlSender<T>,
    ) -> Result<Self::InnerSender<T>, Self::ControlSender<T>>;

    /// Returns the original receiver when it is already wrapped.
    /// This allows attaching metrics only once without losing existing wrappers.
    fn try_into_inner_receiver<T>(
        receiver: Self::ControlReceiver<T>,
    ) -> Result<Self::InnerReceiver<T>, Self::ControlReceiver<T>>;

    /// Wrap a raw sender in the mode-specific control sender type.
    /// Used when no metrics are attached or when rewrapping an already unwrapped channel.
    fn from_inner_sender<T>(sender: Self::InnerSender<T>) -> Self::ControlSender<T>;

    /// Wrap a raw receiver in the mode-specific control receiver type.
    /// Used when no metrics are attached or when rewrapping an already unwrapped channel.
    fn from_inner_receiver<T>(receiver: Self::InnerReceiver<T>) -> Self::ControlReceiver<T>;

    /// Attach metrics to a sender and register it with the channel metrics registry.
    /// Returns the wrapped sender that records send outcomes.
    fn attach_sender_metrics<T>(
        sender: Self::InnerSender<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: MetricSet<ChannelSenderMetrics>,
    ) -> Self::ControlSender<T>;

    /// Attach metrics to a receiver and register it with the channel metrics registry.
    /// Returns the wrapped receiver that records receive outcomes and capacity.
    fn attach_receiver_metrics<T>(
        receiver: Self::InnerReceiver<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: MetricSet<ChannelReceiverMetrics>,
        capacity: u64,
    ) -> Self::ControlReceiver<T>;
}

pub(crate) struct LocalMode;
pub(crate) struct SharedMode;

impl ChannelMode for LocalMode {
    const CHANNEL_MODE: &'static str = CHANNEL_MODE_LOCAL;
    const CHANNEL_IMPL: &'static str = CHANNEL_IMPL_INTERNAL;

    type ControlSender<T> = LocalSender<T>;
    type ControlReceiver<T> = LocalReceiver<T>;
    type InnerSender<T> = mpsc::Sender<T>;
    type InnerReceiver<T> = mpsc::Receiver<T>;

    fn try_into_inner_sender<T>(
        sender: Self::ControlSender<T>,
    ) -> Result<Self::InnerSender<T>, Self::ControlSender<T>> {
        sender.into_mpsc()
    }

    fn try_into_inner_receiver<T>(
        receiver: Self::ControlReceiver<T>,
    ) -> Result<Self::InnerReceiver<T>, Self::ControlReceiver<T>> {
        receiver.into_mpsc()
    }

    fn from_inner_sender<T>(sender: Self::InnerSender<T>) -> Self::ControlSender<T> {
        LocalSender::mpsc(sender)
    }

    fn from_inner_receiver<T>(receiver: Self::InnerReceiver<T>) -> Self::ControlReceiver<T> {
        LocalReceiver::mpsc(receiver)
    }

    fn attach_sender_metrics<T>(
        sender: Self::InnerSender<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: MetricSet<ChannelSenderMetrics>,
    ) -> Self::ControlSender<T> {
        LocalSender::mpsc_with_metrics(sender, channel_metrics, metrics)
    }

    fn attach_receiver_metrics<T>(
        receiver: Self::InnerReceiver<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: MetricSet<ChannelReceiverMetrics>,
        capacity: u64,
    ) -> Self::ControlReceiver<T> {
        LocalReceiver::mpsc_with_metrics(receiver, channel_metrics, metrics, capacity)
    }
}

impl ChannelMode for SharedMode {
    const CHANNEL_MODE: &'static str = CHANNEL_MODE_SHARED;
    const CHANNEL_IMPL: &'static str = CHANNEL_IMPL_TOKIO;

    type ControlSender<T> = SharedSender<T>;
    type ControlReceiver<T> = SharedReceiver<T>;
    type InnerSender<T> = tokio::sync::mpsc::Sender<T>;
    type InnerReceiver<T> = tokio::sync::mpsc::Receiver<T>;

    fn try_into_inner_sender<T>(
        sender: Self::ControlSender<T>,
    ) -> Result<Self::InnerSender<T>, Self::ControlSender<T>> {
        sender.into_mpsc()
    }

    fn try_into_inner_receiver<T>(
        receiver: Self::ControlReceiver<T>,
    ) -> Result<Self::InnerReceiver<T>, Self::ControlReceiver<T>> {
        receiver.into_mpsc()
    }

    fn from_inner_sender<T>(sender: Self::InnerSender<T>) -> Self::ControlSender<T> {
        SharedSender::mpsc(sender)
    }

    fn from_inner_receiver<T>(receiver: Self::InnerReceiver<T>) -> Self::ControlReceiver<T> {
        SharedReceiver::mpsc(receiver)
    }

    fn attach_sender_metrics<T>(
        sender: Self::InnerSender<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: MetricSet<ChannelSenderMetrics>,
    ) -> Self::ControlSender<T> {
        SharedSender::mpsc_with_metrics(sender, channel_metrics, metrics)
    }

    fn attach_receiver_metrics<T>(
        receiver: Self::InnerReceiver<T>,
        channel_metrics: &mut ChannelMetricsRegistry,
        metrics: MetricSet<ChannelReceiverMetrics>,
        capacity: u64,
    ) -> Self::ControlReceiver<T> {
        SharedReceiver::mpsc_with_metrics(receiver, channel_metrics, metrics, capacity)
    }
}

/// Generic helper used by receiver, processor, and exporter wrappers.
/// It keeps local and shared wiring identical while still emitting mode-specific code.
///
/// The logic first attempts to unwrap the inner MPSC channel so metrics can be attached.
/// If the channel is already wrapped, it preserves the existing wrapper to avoid double
/// instrumentation.
pub(crate) fn wrap_control_channel_metrics<M, PData>(
    node_id: &crate::node::NodeId,
    pipeline_ctx: &PipelineContext,
    channel_metrics: &mut ChannelMetricsRegistry,
    channel_metrics_enabled: bool,
    capacity: u64,
    control_sender: M::ControlSender<NodeControlMsg<PData>>,
    control_receiver: M::ControlReceiver<NodeControlMsg<PData>>,
) -> (
    M::ControlSender<NodeControlMsg<PData>>,
    M::ControlReceiver<NodeControlMsg<PData>>,
)
where
    M: ChannelMode,
{
    let control_sender = M::try_into_inner_sender(control_sender);
    let control_receiver = M::try_into_inner_receiver(control_receiver);
    match (control_sender, control_receiver) {
        (Ok(sender), Ok(receiver)) => {
            let channel_entity_key = pipeline_ctx.register_channel_entity(
                control_channel_id(node_id),
                "input".into(),
                CHANNEL_KIND_CONTROL,
                M::CHANNEL_MODE,
                CHANNEL_TYPE_MPSC,
                M::CHANNEL_IMPL,
            );
            if let Some(telemetry) = current_node_telemetry_handle() {
                telemetry.set_control_channel_key(channel_entity_key);
            }
            if channel_metrics_enabled {
                let sender_metrics = pipeline_ctx
                    .register_metric_set_for_entity::<ChannelSenderMetrics>(channel_entity_key);
                let receiver_metrics = pipeline_ctx
                    .register_metric_set_for_entity::<ChannelReceiverMetrics>(channel_entity_key);
                (
                    M::attach_sender_metrics(sender, channel_metrics, sender_metrics),
                    M::attach_receiver_metrics(
                        receiver,
                        channel_metrics,
                        receiver_metrics,
                        capacity,
                    ),
                )
            } else {
                (
                    M::from_inner_sender(sender),
                    M::from_inner_receiver(receiver),
                )
            }
        }
        (sender, receiver) => {
            let sender = match sender {
                Ok(sender) => M::from_inner_sender(sender),
                Err(sender) => sender,
            };
            let receiver = match receiver {
                Ok(receiver) => M::from_inner_receiver(receiver),
                Err(receiver) => receiver,
            };
            (sender, receiver)
        }
    }
}
