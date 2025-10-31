// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for gRPC receivers.

use crate::otap_grpc::GrpcServerConfig;
use crate::otap_grpc::otlp::server::{RouteResponse, SharedState};
use crate::pdata::OtapPdata;
use otap_df_config::experimental::SignalType;
use otap_df_engine::control::{AckMsg, NackMsg};
use tonic::transport::Server;

/// Bundle of shared states for routing acknowledgements.
#[derive(Clone, Default)]
pub struct SignalSharedStates {
    /// Shared state for log acknowledgements.
    pub logs: Option<SharedState>,
    /// Shared state for metric acknowledgements.
    pub metrics: Option<SharedState>,
    /// Shared state for trace acknowledgements.
    pub traces: Option<SharedState>,
}

impl SignalSharedStates {
    /// Creates a new bundle of optional shared states.
    pub fn new(
        logs: Option<SharedState>,
        metrics: Option<SharedState>,
        traces: Option<SharedState>,
    ) -> Self {
        Self {
            logs,
            metrics,
            traces,
        }
    }
}

/// Routes an Ack message to the appropriate shared state.
pub fn route_ack_response(states: &SignalSharedStates, ack: AckMsg<OtapPdata>) -> RouteResponse {
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
pub fn route_nack_response(
    states: &SignalSharedStates,
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

/// Tunes the maximum concurrent requests relative to the downstream capacity.
pub fn tune_max_concurrent_requests(config: &mut GrpcServerConfig, downstream_capacity: usize) {
    // Fall back to the downstream channel capacity when it is tighter than the user setting.
    let safe_capacity = downstream_capacity.max(1);
    if config.max_concurrent_requests == 0 || config.max_concurrent_requests > safe_capacity {
        config.max_concurrent_requests = safe_capacity;
    }
}

/// Applies the shared server tuning options to a tonic server builder.
pub fn apply_server_tuning<L>(builder: Server<L>, config: &GrpcServerConfig) -> Server<L> {
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
