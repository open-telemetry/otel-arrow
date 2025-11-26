// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for gRPC receivers.
//!
//! Request lifecycle (see `otlp::server` for the per-signal services):
//! - decode: gRPC body stays as OTLP bytes, wrapped into `OtapPdata`
//! - subscribe: when `wait_for_result` is enabled, a slot is allocated and calldata recorded
//! - send: payload is forwarded into the pipeline
//! - wait (optional): task awaits an ACK or NACK routed by the subscription maps
//! - respond: ACK/NACK is mapped back to the waiting gRPC request

use crate::otap_grpc::otlp::server::{AckSlot, RouteResponse};
use crate::pdata::OtapPdata;
use otap_df_config::SignalType;
use otap_df_engine::control::{AckMsg, NackMsg};
use tonic::transport::Server;
use crate::otap_grpc::server_settings::GrpcServerSettings;

/// Aggregates the per-signal ACK subscription maps that let us route responses back to callers.
#[derive(Clone, Default)]
pub struct AckRegistry {
    /// Subscription map for log acknowledgements.
    pub logs: Option<AckSlot>,
    /// Subscription map for metric acknowledgements.
    pub metrics: Option<AckSlot>,
    /// Subscription map for trace acknowledgements.
    pub traces: Option<AckSlot>,
}

impl AckRegistry {
    /// Creates a new bundle of optional subscription maps.
    #[must_use]
    pub fn new(
        logs: Option<AckSlot>,
        metrics: Option<AckSlot>,
        traces: Option<AckSlot>,
    ) -> Self {
        Self {
            logs,
            metrics,
            traces,
        }
    }
}

/// Routes an Ack message to the appropriate signal's subscription map.
#[must_use]
pub fn route_ack_response(states: &AckRegistry, ack: AckMsg<OtapPdata>) -> RouteResponse {
    let calldata = ack.calldata;
    let resp = Ok(());
    let state = match ack.accepted.signal_type() {
        SignalType::Logs => states.logs.as_ref(),
        SignalType::Metrics => states.metrics.as_ref(),
        SignalType::Traces => states.traces.as_ref(),
    };

    state
        .map(|s| s.route_response(calldata, resp))
        .unwrap_or(RouteResponse::None)
}

/// Routes a Nack message to the appropriate shared state.
#[must_use]
pub fn route_nack_response(
    states: &AckRegistry,
    mut nack: NackMsg<OtapPdata>,
) -> RouteResponse {
    let calldata = std::mem::take(&mut nack.calldata);
    let signal_type = nack.refused.signal_type();
    let resp = Err(nack);
    let state = match signal_type {
        SignalType::Logs => states.logs.as_ref(),
        SignalType::Metrics => states.metrics.as_ref(),
        SignalType::Traces => states.traces.as_ref(),
    };

    state
        .map(|s| s.route_response(calldata, resp))
        .unwrap_or(RouteResponse::None)
}

/// Handles the outcome from routing an Ack/Nack response.
pub fn handle_route_response<T, F, G>(
    resp: RouteResponse,
    state: &mut T,
    mut on_sent: F,
    mut on_expired_or_invalid: G,
) where
    F: FnMut(&mut T),
    G: FnMut(&mut T),
{
    match resp {
        RouteResponse::Sent => on_sent(state),
        RouteResponse::Expired | RouteResponse::Invalid => on_expired_or_invalid(state),
        RouteResponse::None => {}
    }
}

/// Tunes the maximum concurrent requests relative to the downstream capacity (channel connecting
/// the receiver to the rest of the pipeline).
pub fn tune_max_concurrent_requests(config: &mut GrpcServerSettings, downstream_capacity: usize) {
    // Fall back to the downstream channel capacity when it is tighter than the user setting.
    let safe_capacity = downstream_capacity.max(1);
    if config.max_concurrent_requests == 0 || config.max_concurrent_requests > safe_capacity {
        config.max_concurrent_requests = safe_capacity;
    }
}

/// Applies the shared server tuning options to a tonic server builder.
pub fn apply_server_tuning<L>(builder: Server<L>, config: &GrpcServerSettings) -> Server<L> {
    let transport_limit = config
        .transport_concurrency_limit
        .and_then(|limit| if limit == 0 { None } else { Some(limit) })
        .unwrap_or(config.max_concurrent_requests)
        .max(1);

    let fallback_streams = config.max_concurrent_requests.min(u32::MAX as usize) as u32;

    let mut builder = builder
        .concurrency_limit_per_connection(transport_limit)
        .load_shed(config.load_shed)
        .initial_stream_window_size(config.initial_stream_window_size)
        .initial_connection_window_size(config.initial_connection_window_size)
        .max_frame_size(config.max_frame_size)
        .http2_adaptive_window(Some(config.http2_adaptive_window))
        .http2_keepalive_interval(config.http2_keepalive_interval)
        .http2_keepalive_timeout(config.http2_keepalive_timeout);

    let mut max_concurrent_streams = config
        .max_concurrent_streams
        .map(|value| if value == 0 { fallback_streams } else { value })
        .unwrap_or(fallback_streams);
    if max_concurrent_streams == 0 {
        max_concurrent_streams = 1;
    }
    builder = builder.max_concurrent_streams(Some(max_concurrent_streams));

    // Apply timeout if configured
    if let Some(timeout) = config.timeout {
        builder = builder.timeout(timeout);
    }

    builder
}
