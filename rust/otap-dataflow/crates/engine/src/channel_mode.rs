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
use crate::context::{ExtensionContext, PipelineContext};
use crate::entity_context::{EntityTelemetryHandle, current_node_telemetry_handle};
use crate::local::message::{LocalReceiver, LocalSender};
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_channel::mpsc;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::otel_warn;
use otap_df_telemetry::registry::EntityKey;
use std::borrow::Cow;

pub(crate) trait ChannelMode {
    const CHANNEL_MODE: &'static str;
    const CHANNEL_IMPL: &'static str;

    type ControlSender<T>;
    type ControlReceiver<T>;
    type InnerSender<T>;
    type InnerReceiver<T>;

    /// Unwraps the inner MPSC sender from the mode-specific control sender.
    /// Returns `Err(original)` when the underlying variant is not MPSC
    /// (e.g. MPMC); the caller must then rewrap unchanged.
    fn try_into_inner_sender<T>(
        sender: Self::ControlSender<T>,
    ) -> Result<Self::InnerSender<T>, Self::ControlSender<T>>;

    /// Unwraps the inner MPSC receiver from the mode-specific control receiver.
    /// Returns `Err(original)` when the underlying variant is not MPSC
    /// (e.g. MPMC); the caller must then rewrap unchanged.
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

/// Wraps a node-hosted control channel with metrics if enabled.
///
/// Registers the channel as a node-scoped entity and tracks it on the
/// ambient node telemetry handle so it is cleaned up with the node.
pub(crate) fn wrap_node_control_channel_metrics<M, Msg>(
    name: &str,
    pipeline_ctx: &PipelineContext,
    channel_metrics: &mut ChannelMetricsRegistry,
    channel_metrics_enabled: bool,
    capacity: u64,
    control_sender: M::ControlSender<Msg>,
    control_receiver: M::ControlReceiver<Msg>,
) -> (M::ControlSender<Msg>, M::ControlReceiver<Msg>)
where
    M: ChannelMode,
{
    wrap_control_channel_metrics_inner::<M, Msg>(
        channel_metrics,
        channel_metrics_enabled,
        capacity,
        control_sender,
        control_receiver,
        || {
            let key = pipeline_ctx.register_node_channel_entity(
                control_channel_id(name),
                "input".into(),
                CHANNEL_KIND_CONTROL,
                M::CHANNEL_MODE,
                CHANNEL_TYPE_MPSC,
                M::CHANNEL_IMPL,
            );
            if let Some(telemetry) = current_node_telemetry_handle() {
                telemetry.set_control_channel_key(key);
            }
            key
        },
        // Node version: tracking is handled inside
        // `PipelineContext::register_metric_set_for_entity` via ambient
        // `current_node_telemetry_handle()`.
        |channel_entity_key| {
            (
                pipeline_ctx
                    .register_metric_set_for_entity::<ChannelSenderMetrics>(channel_entity_key),
                pipeline_ctx
                    .register_metric_set_for_entity::<ChannelReceiverMetrics>(channel_entity_key),
            )
        },
    )
}

/// Wraps an extension-hosted control channel with metrics if enabled.
///
/// Registers the channel as an extension-scoped entity and tracks both the
/// entity and the attached metric sets on the provided extension entity handle
/// so everything is cleaned up with the extension.
pub(crate) fn wrap_extension_control_channel_metrics<M, Msg>(
    extension_id: Cow<'static, str>,
    variant: crate::extension::wrapper::ExtensionVariant,
    entity_handle: &EntityTelemetryHandle,
    ext_ctx: &ExtensionContext,
    channel_metrics: &mut ChannelMetricsRegistry,
    channel_metrics_enabled: bool,
    capacity: u64,
    control_sender: M::ControlSender<Msg>,
    control_receiver: M::ControlReceiver<Msg>,
) -> (M::ControlSender<Msg>, M::ControlReceiver<Msg>)
where
    M: ChannelMode,
{
    wrap_control_channel_metrics_inner::<M, Msg>(
        channel_metrics,
        channel_metrics_enabled,
        capacity,
        control_sender,
        control_receiver,
        || {
            let key = ext_ctx.register_extension_channel_entity(
                extension_id.clone(),
                variant,
                control_channel_id(extension_id.as_ref()),
                M::CHANNEL_MODE,
                M::CHANNEL_IMPL,
            );
            entity_handle.track_entity(key);
            key
        },
        |channel_entity_key| {
            (
                entity_handle
                    .register_metric_set_for_entity::<ChannelSenderMetrics>(channel_entity_key),
                entity_handle
                    .register_metric_set_for_entity::<ChannelReceiverMetrics>(channel_entity_key),
            )
        },
    )
}

/// Shared wiring: attempts to unwrap inner MPSC channels, registers the channel
/// entity via `register_channel`, and attaches metrics produced by
/// `register_metrics`. If the channel is already wrapped, preserves the existing
/// wrapper to avoid double instrumentation.
fn wrap_control_channel_metrics_inner<M, Msg>(
    channel_metrics: &mut ChannelMetricsRegistry,
    channel_metrics_enabled: bool,
    capacity: u64,
    control_sender: M::ControlSender<Msg>,
    control_receiver: M::ControlReceiver<Msg>,
    register_channel: impl FnOnce() -> EntityKey,
    register_metrics: impl FnOnce(
        EntityKey,
    ) -> (
        MetricSet<ChannelSenderMetrics>,
        MetricSet<ChannelReceiverMetrics>,
    ),
) -> (M::ControlSender<Msg>, M::ControlReceiver<Msg>)
where
    M: ChannelMode,
{
    let control_sender = M::try_into_inner_sender(control_sender);
    let control_receiver = M::try_into_inner_receiver(control_receiver);
    match (control_sender, control_receiver) {
        (Ok(sender), Ok(receiver)) => {
            let channel_entity_key = register_channel();
            if channel_metrics_enabled {
                let (sender_metrics, receiver_metrics) = register_metrics(channel_entity_key);
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
            // Mismatched halves: structurally impossible when both come
            // from the same factory; surface loudly rather than silently
            // skipping metric registration.
            let sender_was_inner = sender.is_ok();
            let receiver_was_inner = receiver.is_ok();
            debug_assert!(
                false,
                "wrap_control_channel_metrics_inner: mismatched partial_wrap inputs \
                 (sender_was_inner={sender_was_inner}, receiver_was_inner={receiver_was_inner}); \
                 channel sender and receiver must originate from the same factory",
            );
            otel_warn!(
                "channel.metrics.partial_wrap_skip",
                sender_was_inner = sender_was_inner,
                receiver_was_inner = receiver_was_inner,
                message = "mismatched halves; skipping metric registration",
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_channel::mpmc as raw_mpmc;
    use otap_df_channel::mpsc as raw_mpsc;
    use std::cell::Cell;
    use std::num::NonZeroUsize;
    use std::rc::Rc;

    fn mismatched_local_pair<T: 'static>() -> (LocalSender<T>, LocalReceiver<T>) {
        let (mpsc_sender, _mpsc_receiver) = raw_mpsc::Channel::<T>::new(4);
        let (_mpmc_sender, mpmc_receiver) =
            raw_mpmc::Channel::<T>::new(NonZeroUsize::new(4).expect("non-zero capacity"));
        (
            LocalSender::mpsc(mpsc_sender),
            LocalReceiver::mpmc(mpmc_receiver),
        )
    }

    #[test]
    #[should_panic(expected = "partial_wrap")]
    fn partial_wrap_panics_in_debug_builds() {
        let (sender, receiver) = mismatched_local_pair::<u8>();
        let mut channel_metrics = ChannelMetricsRegistry::default();
        let _ = wrap_control_channel_metrics_inner::<LocalMode, u8>(
            &mut channel_metrics,
            true,
            4,
            sender,
            receiver,
            || panic!("register_channel must not be invoked on the partial-wrap path"),
            |_key| panic!("register_metrics must not be invoked on the partial-wrap path"),
        );
    }

    #[test]
    fn partial_wrap_does_not_register_or_attach_metrics() {
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let register_invocations: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let metrics_invocations: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let r_invocations = Rc::clone(&register_invocations);
        let m_invocations = Rc::clone(&metrics_invocations);

        let mut channel_metrics = ChannelMetricsRegistry::default();
        let (sender, receiver) = mismatched_local_pair::<u8>();

        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _ = wrap_control_channel_metrics_inner::<LocalMode, u8>(
                &mut channel_metrics,
                true,
                4,
                sender,
                receiver,
                move || {
                    r_invocations.set(r_invocations.get() + 1);
                    EntityKey::default()
                },
                move |_key| {
                    m_invocations.set(m_invocations.get() + 1);
                    unreachable!("metrics registration must not run when partial-wrap is detected");
                },
            );
        }));
        std::panic::set_hook(prev_hook);

        assert_eq!(
            register_invocations.get(),
            0,
            "partial-wrap must not register a channel entity",
        );
        assert_eq!(
            metrics_invocations.get(),
            0,
            "partial-wrap must not attach metric handles",
        );
        assert!(
            channel_metrics.into_handles().is_empty(),
            "channel metrics registry must remain empty after a partial-wrap call",
        );
    }
}
